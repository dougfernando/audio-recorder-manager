#[cfg(windows)]
pub mod windows_monitor {
    use anyhow::{Context, Result};
    use crate::audio_utils::{calculate_rms_f32, calculate_rms_i16, calculate_rms_i32};
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;

    const REFTIMES_PER_SEC: i64 = 10_000_000;

    pub struct AudioLevelMonitor {
        is_monitoring: Arc<AtomicBool>,
        loopback_level: Arc<AtomicU32>, // Store as u32 (f32 bits)
        microphone_level: Arc<AtomicU32>,
    }

    impl AudioLevelMonitor {
        pub fn new() -> Result<Self> {
            let is_monitoring = Arc::new(AtomicBool::new(true));
            let loopback_level = Arc::new(AtomicU32::new(0));
            let microphone_level = Arc::new(AtomicU32::new(0));

            // Start loopback monitoring thread
            let is_monitoring_clone = Arc::clone(&is_monitoring);
            let loopback_level_clone = Arc::clone(&loopback_level);
            std::thread::spawn(move || {
                let _ = Self::monitor_loopback(is_monitoring_clone, loopback_level_clone);
            });

            // Start microphone monitoring thread
            let is_monitoring_clone = Arc::clone(&is_monitoring);
            let microphone_level_clone = Arc::clone(&microphone_level);
            std::thread::spawn(move || {
                let _ = Self::monitor_microphone(is_monitoring_clone, microphone_level_clone);
            });

            Ok(Self {
                is_monitoring,
                loopback_level,
                microphone_level,
            })
        }

        fn monitor_loopback(is_monitoring: Arc<AtomicBool>, level: Arc<AtomicU32>) -> Result<()> {
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .context("Failed to initialize COM")?;

                let device_enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

                let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

                let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
                let mix_format = audio_client.GetMixFormat()?;

                audio_client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_LOOPBACK,
                    REFTIMES_PER_SEC,
                    0,
                    mix_format,
                    None,
                )?;

                let capture_client: IAudioCaptureClient = audio_client.GetService()?;
                audio_client.Start()?;

                let wf = &*mix_format;
                let bits_per_sample = wf.wBitsPerSample;
                let format_tag = wf.wFormatTag;
                let channels = wf.nChannels;
                let is_float = (format_tag == 0xFFFE && bits_per_sample == 32) || format_tag == 3;
                let is_32bit = bits_per_sample == 32;

                while is_monitoring.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(50));

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
                            let is_silent = (flags & 0x2) != 0;

                            let rms = if is_silent {
                                0.0
                            } else if is_float && is_32bit {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const f32,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_f32(samples)
                            } else if !is_float && is_32bit {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const i32,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_i32(samples) / 2147483647.0 // Normalize to 0-1
                            } else if bits_per_sample == 16 {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const i16,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_i16(samples) / 32767.0 // Normalize to 0-1
                            } else {
                                0.0
                            };

                            level.store(rms.to_bits(), Ordering::Relaxed);
                        }

                        capture_client.ReleaseBuffer(num_frames)?;
                    }
                }

                audio_client.Stop()?;
                CoTaskMemFree(Some(mix_format as *const _ as *const _));
                CoUninitialize();

                Ok(())
            }
        }

        fn monitor_microphone(is_monitoring: Arc<AtomicBool>, level: Arc<AtomicU32>) -> Result<()> {
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)
                    .ok()
                    .context("Failed to initialize COM")?;

                let device_enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

                let device = device_enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;

                let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
                let mix_format = audio_client.GetMixFormat()?;

                audio_client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    0,
                    REFTIMES_PER_SEC,
                    0,
                    mix_format,
                    None,
                )?;

                let capture_client: IAudioCaptureClient = audio_client.GetService()?;
                audio_client.Start()?;

                let wf = &*mix_format;
                let bits_per_sample = wf.wBitsPerSample;
                let format_tag = wf.wFormatTag;
                let channels = wf.nChannels;
                let is_float = (format_tag == 0xFFFE && bits_per_sample == 32) || format_tag == 3;
                let is_32bit = bits_per_sample == 32;

                while is_monitoring.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(50));

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
                            let is_silent = (flags & 0x2) != 0;

                            let rms = if is_silent {
                                0.0
                            } else if is_float && is_32bit {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const f32,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_f32(samples)
                            } else if !is_float && is_32bit {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const i32,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_i32(samples) / 2147483647.0 // Normalize to 0-1
                            } else if bits_per_sample == 16 {
                                let samples = std::slice::from_raw_parts(
                                    buffer as *const i16,
                                    (num_frames * channels as u32) as usize,
                                );
                                calculate_rms_i16(samples) / 32767.0 // Normalize to 0-1
                            } else {
                                0.0
                            };

                            level.store(rms.to_bits(), Ordering::Relaxed);
                        }

                        capture_client.ReleaseBuffer(num_frames)?;
                    }
                }

                audio_client.Stop()?;
                CoTaskMemFree(Some(mix_format as *const _ as *const _));
                CoUninitialize();

                Ok(())
            }
        }

        pub fn get_loopback_level(&self) -> f32 {
            let bits = self.loopback_level.load(Ordering::Relaxed);
            f32::from_bits(bits)
        }

        pub fn get_microphone_level(&self) -> f32 {
            let bits = self.microphone_level.load(Ordering::Relaxed);
            f32::from_bits(bits)
        }

        pub fn stop(&self) {
            self.is_monitoring.store(false, Ordering::Relaxed);
        }
    }

    impl Drop for AudioLevelMonitor {
        fn drop(&mut self) {
            self.stop();
        }
    }

}
