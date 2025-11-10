#[cfg(windows)]
pub mod windows_microphone {
    use anyhow::{Context, Result};
    use hound::{WavSpec, WavWriter};
    use std::path::PathBuf;
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
        sample_rate: u32,
        channels: u16,
    }

    impl WasapiMicrophoneRecorder {
        pub fn new(filepath: PathBuf) -> Result<Self> {
            let is_recording = Arc::new(AtomicBool::new(true));
            let frames_captured = Arc::new(AtomicU64::new(0));
            let has_audio = Arc::new(AtomicBool::new(false));

            let is_recording_clone = Arc::clone(&is_recording);
            let frames_captured_clone = Arc::clone(&frames_captured);
            let has_audio_clone = Arc::clone(&has_audio);

            // Get sample rate and channels before spawning thread
            let (sample_rate, channels) = unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .context("Failed to initialize COM")?;

                let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(
                    &MMDeviceEnumerator,
                    None,
                    CLSCTX_ALL,
                )?;

                let device = device_enumerator
                    .GetDefaultAudioEndpoint(eCapture, eConsole)?;

                let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
                let mix_format = audio_client.GetMixFormat()?;
                let wf = &*mix_format;

                let sr = wf.nSamplesPerSec;
                let ch = wf.nChannels;

                CoTaskMemFree(Some(mix_format as *const _ as *const _));

                (sr, ch)
            };

            // Spawn recording thread that initializes its own COM
            std::thread::spawn(move || {
                let _ = Self::recording_thread(
                    filepath,
                    is_recording_clone,
                    frames_captured_clone,
                    has_audio_clone,
                );
            });

            Ok(Self {
                is_recording,
                frames_captured,
                has_audio,
                sample_rate,
                channels,
            })
        }

        fn recording_thread(
            filepath: PathBuf,
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
                let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(
                    &MMDeviceEnumerator,
                    None,
                    CLSCTX_ALL,
                )?;

                let device = device_enumerator
                    .GetDefaultAudioEndpoint(eCapture, eConsole)?;

                // Activate audio client
                let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

                // Get mix format
                let mix_format = audio_client.GetMixFormat()?;
                let wf = &*mix_format;

                // Copy packed struct fields to avoid alignment issues
                let sample_rate = wf.nSamplesPerSec;
                let channels = wf.nChannels;
                let bits_per_sample = wf.wBitsPerSample;
                let format_tag = wf.wFormatTag;

                // WASAPI usually returns WAVEFORMATEXTENSIBLE (0xFFFE) for modern formats
                // We need to check the SubFormat GUID to determine actual format
                let is_float = if format_tag == 0xFFFE {  // WAVE_FORMAT_EXTENSIBLE
                    // For WAVEFORMATEXTENSIBLE, check SubFormat GUID
                    // KSDATAFORMAT_SUBTYPE_IEEE_FLOAT ends with 00 00 10 00 ...
                    // KSDATAFORMAT_SUBTYPE_PCM ends with 01 00 10 00 ...
                    // The format is typically 32-bit float for capture
                    bits_per_sample == 32  // Assume 32-bit is float in capture
                } else {
                    format_tag == 3  // WAVE_FORMAT_IEEE_FLOAT
                };

                log::info!(
                    "WASAPI Microphone format: {} Hz, {} channels, {} bits, format tag: 0x{:X}, float: {}",
                    sample_rate,
                    channels,
                    bits_per_sample,
                    format_tag,
                    is_float
                );

                // Initialize audio client in capture mode (no loopback flag)
                audio_client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    0, // No loopback flag for microphone capture
                    REFTIMES_PER_SEC, // 1 second buffer
                    0,
                    mix_format,
                    None,
                )?;

                // Get capture client
                let capture_client: IAudioCaptureClient = audio_client.GetService()?;

                // Create WAV writer - always use 16-bit PCM for compatibility
                let spec = WavSpec {
                    channels,
                    sample_rate,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };

                let writer = WavWriter::create(&filepath, spec)?;
                let mut writer = Some(writer);

                // Start audio client
                audio_client.Start()?;

                let is_32bit = bits_per_sample == 32;
                log::info!("Recording microphone with: {} bit, {}", bits_per_sample, if is_float { "float" } else { "int" });

                // Recording loop
                while is_recording.load(Ordering::Relaxed) {
                    // Wait a bit for buffer to fill
                    std::thread::sleep(Duration::from_millis(10));

                    let packet_length = capture_client.GetNextPacketSize()?;

                    if packet_length > 0 {
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
                                if let Some(writer) = writer.as_mut() {
                                    for _ in 0..(num_frames * channels as u32) {
                                        let _ = writer.write_sample(0i16);
                                    }
                                }
                            } else {
                                // Process based on actual format
                                if is_float && is_32bit {
                                    // 32-bit float samples
                                    let samples = std::slice::from_raw_parts(
                                        buffer as *const f32,
                                        (num_frames * channels as u32) as usize,
                                    );

                                    // Detect audio
                                    if !has_audio.load(Ordering::Relaxed) {
                                        let rms = calculate_rms_f32(samples);
                                        if rms > 0.01 {
                                            has_audio.store(true, Ordering::Relaxed);
                                            log::info!("Microphone audio detected! Level: {:.4}", rms);
                                        }
                                    }

                                    // Convert and write to 16-bit
                                    if let Some(writer) = writer.as_mut() {
                                        for &sample in samples {
                                            let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                                            let _ = writer.write_sample(sample_i16);
                                        }
                                    }
                                } else if !is_float && is_32bit {
                                    // 32-bit int samples
                                    let samples = std::slice::from_raw_parts(
                                        buffer as *const i32,
                                        (num_frames * channels as u32) as usize,
                                    );

                                    // Detect audio
                                    if !has_audio.load(Ordering::Relaxed) {
                                        let rms = calculate_rms_i32(samples);
                                        if rms > 100000.0 {  // Adjusted threshold for 32-bit
                                            has_audio.store(true, Ordering::Relaxed);
                                            log::info!("Microphone audio detected! Level: {:.2}", rms);
                                        }
                                    }

                                    // Convert and write to 16-bit
                                    // Scale from 32-bit range to 16-bit range
                                    // i32 range: -2147483648 to 2147483647
                                    // i16 range: -32768 to 32767
                                    // Shift right by 16 bits (equivalent to dividing by 65536)
                                    if let Some(writer) = writer.as_mut() {
                                        for &sample in samples {
                                            let sample_i16 = (sample >> 16) as i16;
                                            let _ = writer.write_sample(sample_i16);
                                        }
                                    }
                                } else if !is_float && bits_per_sample == 16 {
                                    // 16-bit int samples (legacy)
                                    let samples = std::slice::from_raw_parts(
                                        buffer as *const i16,
                                        (num_frames * channels as u32) as usize,
                                    );

                                    // Detect audio
                                    if !has_audio.load(Ordering::Relaxed) {
                                        let rms = calculate_rms_i16(samples);
                                        if rms > 100.0 {
                                            has_audio.store(true, Ordering::Relaxed);
                                            log::info!("Microphone audio detected! Level: {:.2}", rms);
                                        }
                                    }

                                    // Write directly
                                    if let Some(writer) = writer.as_mut() {
                                        for &sample in samples {
                                            let _ = writer.write_sample(sample);
                                        }
                                    }
                                } else {
                                    log::warn!("Unsupported microphone audio format: {} bits, float={}", bits_per_sample, is_float);
                                }
                            }
                        }

                        capture_client.ReleaseBuffer(num_frames)?;
                    }
                }

                // Stop and cleanup
                audio_client.Stop()?;

                if let Some(writer) = writer.take() {
                    writer.finalize()?;
                }

                CoTaskMemFree(Some(mix_format as *const _ as *const _));
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

        pub fn get_sample_rate(&self) -> u32 {
            self.sample_rate
        }

        pub fn get_channels(&self) -> u16 {
            self.channels
        }

        pub fn stop(&self) -> Result<()> {
            self.is_recording.store(false, Ordering::Relaxed);

            // Wait a moment for recording thread to finish
            std::thread::sleep(Duration::from_millis(1000));

            Ok(())
        }
    }

    fn calculate_rms_i16(samples: &[i16]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }

    fn calculate_rms_i32(samples: &[i32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }

    fn calculate_rms_f32(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        (sum / samples.len() as f64).sqrt() as f32
    }
}

#[cfg(not(windows))]
pub mod windows_microphone {
    use anyhow::Result;
    use std::path::PathBuf;

    pub struct WasapiMicrophoneRecorder;

    impl WasapiMicrophoneRecorder {
        pub fn new(_filepath: PathBuf) -> Result<Self> {
            anyhow::bail!("WASAPI microphone is only available on Windows")
        }

        pub fn get_frames_captured(&self) -> u64 {
            0
        }

        pub fn has_audio_detected(&self) -> bool {
            false
        }

        pub fn get_sample_rate(&self) -> u32 {
            48000
        }

        pub fn get_channels(&self) -> u16 {
            2
        }

        pub fn stop(&self) -> Result<()> {
            Ok(())
        }
    }
}
