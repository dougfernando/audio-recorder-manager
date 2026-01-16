#[cfg(windows)]
pub mod windows_microphone {
    //! Windows WASAPI microphone recording implementation
    //!
    //! # Safety
    //!
    //! This module uses extensive `unsafe` code to interact with the Windows Audio Session API (WASAPI).
    //! All unsafe operations follow the same safety invariants as the loopback module:
    //!
    //! - **COM Initialization**: Proper initialization and cleanup on each thread
    //! - **Pointer Validity**: All WASAPI pointers are used only while valid
    //! - **Buffer Access**: Audio buffers are validated before casting to typed slices
    //! - **Memory Layout**: Relies on WASAPI's alignment guarantees for audio buffers

    use anyhow::{Context, Result};
    use crate::audio_utils::{calculate_rms_f32, calculate_rms_i16};
    use crate::streaming_encoder::StreamingEncoder;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;

    const REFTIMES_PER_SEC: i64 = 10_000_000;

    pub struct WasapiMicrophoneRecorder {
        is_recording: Arc<AtomicBool>,
        frames_captured: Arc<AtomicU64>,
        has_audio: Arc<AtomicBool>,
    }

    impl WasapiMicrophoneRecorder {
        pub fn new(mut encoder: Box<dyn StreamingEncoder>, target_sample_rate: u32) -> Result<Self> {
            let is_recording = Arc::new(AtomicBool::new(true));
            let frames_captured = Arc::new(AtomicU64::new(0));
            let has_audio = Arc::new(AtomicBool::new(false));

            let is_recording_clone = Arc::clone(&is_recording);
            let frames_captured_clone = Arc::clone(&frames_captured);
            let has_audio_clone = Arc::clone(&has_audio);

            // Spawn recording thread that initializes its own COM
            std::thread::spawn(move || {
                if let Err(e) = Self::recording_thread(
                    encoder,
                    target_sample_rate,
                    is_recording_clone,
                    frames_captured_clone,
                    has_audio_clone,
                ) {
                    tracing::error!("Microphone recording thread error: {}", e);
                    tracing::error!("Error chain: {:?}", e);
                }
            });

            Ok(Self {
                is_recording,
                frames_captured,
                has_audio,
            })
        }

        fn recording_thread(
            mut encoder: Box<dyn StreamingEncoder>,
            target_sample_rate: u32,
            is_recording: Arc<AtomicBool>,
            frames_captured: Arc<AtomicU64>,
            has_audio: Arc<AtomicBool>,
        ) -> Result<()> {
            unsafe {
                // Initialize COM for this thread
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .context("Failed to initialize COM in recording thread")?;

                // Get default audio endpoint for capture (microphone/input)
                let device_enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

                let device = device_enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;

                // Activate audio client
                let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

                // Get microphone's native format
                let native_format = audio_client.GetMixFormat()?;
                let native_wf = &*native_format;

                // Copy packed struct fields to avoid alignment issues
                let channels = native_wf.nChannels;
                let native_sample_rate = native_wf.nSamplesPerSec;
                let bits_per_sample = native_wf.wBitsPerSample;
                let format_tag = native_wf.wFormatTag;

                tracing::info!(
                    "WASAPI Microphone native format: {} Hz, {} channels, {} bits, tag: {}",
                    native_sample_rate,
                    channels,
                    bits_per_sample,
                    format_tag
                );

                // IMPORTANT: Use native format to avoid AUDCLNT_E_UNSUPPORTED_FORMAT errors.
                // FFmpeg will handle resampling during the merge to match system audio.
                tracing::info!(
                    "Using native microphone format - FFmpeg will resample to {} Hz during merge",
                    target_sample_rate
                );

                // Initialize audio client with native format
                tracing::info!("Initializing audio client with native format...");
                audio_client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    0,                // No loopback flag for microphone capture
                    REFTIMES_PER_SEC, // 1 second buffer
                    0,
                    native_format,
                    None,
                )?;
                tracing::info!("Audio client initialized successfully");

                // Free native format (we're done with it after Initialize)
                CoTaskMemFree(Some(native_format as *const _ as *const _));

                // Record at native sample rate - FFmpeg will resample during merge
                let sample_rate = native_sample_rate;

                // Detect if using float format
                // Format tag 3 = WAVE_FORMAT_IEEE_FLOAT, 0xFFFE = WAVE_FORMAT_EXTENSIBLE (check subformat)
                let is_float = format_tag == 3 || (format_tag == 0xFFFE && bits_per_sample == 32);

                // Get capture client
                tracing::info!("Getting capture client service...");
                let capture_client: IAudioCaptureClient = audio_client.GetService()?;
                tracing::info!("Capture client obtained successfully");

                // Start audio client
                tracing::info!("Starting microphone audio client...");
                audio_client.Start()?;
                tracing::info!("Audio client started successfully");

                tracing::info!(
                    "Recording microphone: {} Hz, {} channels, {} bits, {} format",
                    sample_rate,
                    channels,
                    bits_per_sample,
                    if is_float { "float" } else { "int" }
                );
                tracing::info!("Microphone audio client started, entering recording loop...");

                // Recording loop
                let mut loop_count = 0u64;
                let mut total_packets_received = 0u64;
                while is_recording.load(Ordering::Relaxed) {
                    // Wait a bit for buffer to fill
                    std::thread::sleep(Duration::from_millis(10));

                    let packet_length = capture_client.GetNextPacketSize()?;

                    // Log every 100 iterations (~1 second) to track packet availability
                    if loop_count % 100 == 0 {
                        tracing::debug!(
                            "Mic loop #{}: packet_length={}, total_packets={}",
                            loop_count,
                            packet_length,
                            total_packets_received
                        );
                    }
                    loop_count += 1;

                    if packet_length > 0 {
                        total_packets_received += 1;
                        tracing::debug!("Mic packet available: {} frames", packet_length);
                        let mut buffer: *mut u8 = std::ptr::null_mut();
                        let mut num_frames = 0u32;
                        let mut flags = 0u32;

                        capture_client.GetBuffer(
                            &mut buffer,
                            &mut num_frames,
                            &mut flags,
                            None,
                            None,
                        )?;

                        if num_frames > 0 {
                            frames_captured.fetch_add(1, Ordering::Relaxed);

                            // Check if buffer contains silence (AUDCLNT_BUFFERFLAGS_SILENT = 0x2)
                            let is_silent = (flags & 0x2) != 0;

                            if is_silent {
                                // Write silence for the number of frames
                                let silence: Vec<i16> = vec![0i16; (num_frames * channels as u32) as usize];
                                let _ = encoder.write_samples(&silence, channels as u32, sample_rate);
                            } else {
                                // Process audio based on native format
                                if is_float {
                                    // Process 32-bit float samples
                                    let samples = std::slice::from_raw_parts(
                                        buffer as *const f32,
                                        (num_frames * channels as u32) as usize,
                                    );

                                    // Detect audio
                                    if !has_audio.load(Ordering::Relaxed) {
                                        let rms = calculate_rms_f32(samples);
                                        if rms > 0.01 {
                                            has_audio.store(true, Ordering::Relaxed);
                                            tracing::info!(
                                                "Microphone audio detected! Level: {:.4}",
                                                rms
                                            );
                                        }
                                    }

                                    // Convert and write to 16-bit
                                    let samples_i16: Vec<i16> = samples
                                        .iter()
                                        .map(|&sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16)
                                        .collect();
                                    let _ = encoder.write_samples(&samples_i16, channels as u32, sample_rate);
                                } else if bits_per_sample == 16 {
                                    // Process 16-bit int samples
                                    let samples = std::slice::from_raw_parts(
                                        buffer as *const i16,
                                        (num_frames * channels as u32) as usize,
                                    );

                                    // Detect audio
                                    if !has_audio.load(Ordering::Relaxed) {
                                        let rms = calculate_rms_i16(samples);
                                        if rms > 327.0 {
                                            // 0.01 * 32767
                                            has_audio.store(true, Ordering::Relaxed);
                                            tracing::info!(
                                                "Microphone audio detected! Level: {:.4}",
                                                rms / 32767.0
                                            );
                                        }
                                    }

                                    // Write directly
                                    let _ = encoder.write_samples(samples, channels as u32, sample_rate);
                                } else {
                                    // Unsupported format - write silence
                                    tracing::warn!(
                                        "Unsupported audio format: {} bits, writing silence",
                                        bits_per_sample
                                    );
                                    let silence: Vec<i16> = vec![0i16; (num_frames * channels as u32) as usize];
                                    let _ = encoder.write_samples(&silence, channels as u32, sample_rate);
                                }
                            }
                        }

                        capture_client.ReleaseBuffer(num_frames)?;
                    }
                }

                // Stop and cleanup
                tracing::info!(
                    "Microphone recording loop finished: {} iterations, {} packets received, {} frames captured",
                    loop_count,
                    total_packets_received,
                    frames_captured.load(Ordering::Relaxed)
                );

                audio_client.Stop()?;

                // Finalize encoder
                encoder.finish()?;

                // Native format was already freed after Initialize()

                CoUninitialize();

                Ok(())
            }
        }

        pub fn get_frames_captured(&self) -> u64 {
            self.frames_captured.load(Ordering::Relaxed)
        }

        pub fn has_audio_detected(&self) -> bool {
            self.has_audio.load(Ordering::Relaxed)
        }

        pub fn stop(&self) -> Result<()> {
            self.is_recording.store(false, Ordering::Relaxed);

            // Wait a moment for recording thread to finish
            std::thread::sleep(Duration::from_millis(1000));

            Ok(())
        }
    }

}
