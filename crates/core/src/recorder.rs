use anyhow::Result;
#[cfg(not(windows))]
use cpal::traits::{DeviceTrait, StreamTrait};
#[cfg(not(windows))]
use cpal::{Device, SampleFormat, Stream};
#[cfg(not(windows))]
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
#[cfg(not(windows))]
use std::fs::File;
#[cfg(not(windows))]
use std::io::BufWriter;
use std::path::PathBuf;
#[cfg(not(windows))]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(not(windows))]
use std::sync::Mutex;
#[cfg(not(windows))]
use std::time::{Duration, Instant};

const PROFESSIONAL_SAMPLE_RATE: u32 = 48000;
const PROFESSIONAL_CHANNELS: u16 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingQuality {
    pub name: String,
    pub description: String,
    pub size_per_min: String,
    pub sample_rate: u32,
    pub channels: u16,
}

impl RecordingQuality {
    pub fn professional() -> Self {
        Self {
            name: "Professional (48kHz Stereo)".to_string(),
            description: "Professional quality for meetings".to_string(),
            size_per_min: "11 MB/min".to_string(),
            sample_rate: PROFESSIONAL_SAMPLE_RATE,
            channels: PROFESSIONAL_CHANNELS,
        }
    }

    pub fn quick() -> Self {
        Self {
            name: "Quick (16kHz Mono)".to_string(),
            description: "Smaller files, good for voice notes".to_string(),
            size_per_min: "2 MB/min".to_string(),
            sample_rate: 16000,
            channels: 1,
        }
    }

    pub fn standard() -> Self {
        Self {
            name: "Standard (44.1kHz Stereo)".to_string(),
            description: "CD quality, balanced file size".to_string(),
            size_per_min: "10 MB/min".to_string(),
            sample_rate: 44100,
            channels: 2,
        }
    }

    pub fn high() -> Self {
        Self {
            name: "High (96kHz Stereo)".to_string(),
            description: "Maximum quality, larger files".to_string(),
            size_per_min: "22 MB/min".to_string(),
            sample_rate: 96000,
            channels: 2,
        }
    }
}

#[cfg(not(windows))]
pub struct AudioRecorder {
    device: Device,
    device_name: String,
    sample_rate: u32,
    channels: u16,
    output_dir: PathBuf,
}

#[cfg(not(windows))]
impl AudioRecorder {
    pub fn new(device: Device, device_name: String, output_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            device,
            device_name,
            sample_rate: PROFESSIONAL_SAMPLE_RATE,
            channels: PROFESSIONAL_CHANNELS,
            output_dir,
        })
    }

    pub fn with_quality(mut self, quality: &RecordingQuality) -> Self {
        self.sample_rate = quality.sample_rate;
        self.channels = quality.channels;
        self
    }

    pub fn get_device_name(&self) -> &str {
        &self.device_name
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    pub async fn start_recording(
        &self,
        filename: &str,
        duration: Option<u64>,
        session_id: String,
        status_dir: PathBuf,
    ) -> Result<RecordingHandle> {
        std::fs::create_dir_all(&self.output_dir)?;
        std::fs::create_dir_all(&status_dir)?;

        let filepath = self.output_dir.join(filename);
        let status_file = status_dir.join(format!("{}.json", session_id));

        // Get supported config from device
        let supported_config = self.device.default_input_config()?;

        // Use device's default configuration to ensure compatibility
        let config = supported_config.config();

        // Update our internal config to match what the device supports
        let actual_sample_rate = config.sample_rate.0;
        let actual_channels = config.channels;

        let spec = WavSpec {
            channels: actual_channels,
            sample_rate: actual_sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&filepath, spec)?;
        let writer = Arc::new(Mutex::new(Some(writer)));

        let writer_clone = Arc::clone(&writer);
        let is_recording = Arc::new(AtomicBool::new(true));
        let is_recording_clone = Arc::clone(&is_recording);
        let frames_captured = Arc::new(AtomicU64::new(0));
        let frames_captured_clone = Arc::clone(&frames_captured);
        let has_audio = Arc::new(AtomicBool::new(false));
        let has_audio_clone = Arc::clone(&has_audio);

        let err_fn = |err| tracing::error!("Stream error: {}", err);

        // Build the stream based on the supported sample format
        let stream = match supported_config.sample_format() {
            SampleFormat::I16 => self.device.build_input_stream(
                &config,
                move |data: &[i16], _: &_| {
                    if !is_recording_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    frames_captured_clone.fetch_add(1, Ordering::Relaxed);

                    // Detect audio
                    if !has_audio_clone.load(Ordering::Relaxed) {
                        let rms = calculate_rms_i16(data);
                        if rms > 100.0 {
                            has_audio_clone.store(true, Ordering::Relaxed);
                            tracing::info!("Audio detected! Level: {:.2}", rms);
                        }
                    }

                    let mut writer_guard = writer_clone.lock().unwrap();
                    if let Some(writer) = writer_guard.as_mut() {
                        for &sample in data {
                            let _ = writer.write_sample(sample);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::F32 => self.device.build_input_stream(
                &config,
                move |data: &[f32], _: &_| {
                    if !is_recording_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    frames_captured_clone.fetch_add(1, Ordering::Relaxed);

                    // Detect audio
                    if !has_audio_clone.load(Ordering::Relaxed) {
                        let rms = calculate_rms_f32(data);
                        if rms > 0.01 {
                            has_audio_clone.store(true, Ordering::Relaxed);
                            tracing::info!("Audio detected! Level: {:.4}", rms);
                        }
                    }

                    let mut writer_guard = writer_clone.lock().unwrap();
                    if let Some(writer) = writer_guard.as_mut() {
                        for &sample in data {
                            let sample_i16 = (sample * i16::MAX as f32) as i16;
                            let _ = writer.write_sample(sample_i16);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;

        let handle = RecordingHandle {
            stream,
            writer,
            is_recording,
            frames_captured,
            has_audio,
            filepath,
            status_file,
            session_id,
            duration,
            start_time: Instant::now(),
            device_name: self.device_name.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
        };

        Ok(handle)
    }
}

#[cfg(not(windows))]
pub struct RecordingHandle {
    stream: Stream,
    writer: Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    is_recording: Arc<AtomicBool>,
    frames_captured: Arc<AtomicU64>,
    has_audio: Arc<AtomicBool>,
    filepath: PathBuf,
    status_file: PathBuf,
    session_id: String,
    duration: Option<u64>,
    start_time: Instant,
    device_name: String,
    sample_rate: u32,
    channels: u16,
}

// Stream is not Send on Windows but we need it for tokio::spawn
// This is safe because we only access it in the async context
#[cfg(not(windows))]
unsafe impl Send for RecordingHandle {}

#[cfg(not(windows))]
impl RecordingHandle {
    pub fn get_elapsed(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn get_frames_captured(&self) -> u64 {
        self.frames_captured.load(Ordering::Relaxed)
    }

    pub fn has_audio_detected(&self) -> bool {
        self.has_audio.load(Ordering::Relaxed)
    }

    pub fn get_status(&self) -> RecordingStatus {
        let elapsed = self.get_elapsed();
        let duration_secs = self.duration.unwrap_or(0);
        let progress = if duration_secs > 0 {
            ((elapsed as f64 / duration_secs as f64) * 100.0).min(100.0) as u8
        } else {
            0
        };

        RecordingStatus {
            status: "recording".to_string(),
            session_id: self.session_id.clone(),
            filename: self
                .filepath
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            duration: duration_secs,
            elapsed,
            progress,
            quality: "professional".to_string(),
            device: self.device_name.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            frames_captured: self.get_frames_captured(),
            has_audio: self.has_audio_detected(),
            // Per-channel data (single channel recorder doesn't have this)
            loopback_frames: None,
            loopback_has_audio: None,
            mic_frames: None,
            mic_has_audio: None,
            // FFmpeg progress (not applicable during recording)
            ffmpeg_progress: None,
            processing_speed: None,
        }
    }

    pub fn write_status(&self) -> Result<()> {
        let status = self.get_status();
        let json = serde_json::to_string_pretty(&status)?;
        std::fs::write(&self.status_file, json)?;
        Ok(())
    }

    pub fn should_stop(&self) -> bool {
        if let Some(duration) = self.duration {
            if self.get_elapsed() >= duration {
                return true;
            }
        }

        // Check for stop signal
        let signals_dir = self
            .status_file
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("signals"));

        if let Some(signals_dir) = signals_dir {
            let stop_signal = signals_dir.join(format!("{}.stop", self.session_id));
            if stop_signal.exists() {
                tracing::info!("Stop signal received for session {}", self.session_id);
                let _ = std::fs::remove_file(stop_signal);
                return true;
            }
        }

        false
    }

    pub async fn stop(self) -> Result<PathBuf> {
        tracing::info!("Stopping recording...");
        self.is_recording.store(false, Ordering::Relaxed);

        // Get values before dropping stream
        let frames_captured = self.get_frames_captured();
        let has_audio = self.has_audio_detected();
        let filepath = self.filepath.clone();
        let status_file = self.status_file.clone();
        let session_id = self.session_id.clone();
        let duration = self.duration;
        let device_name = self.device_name.clone();
        let sample_rate = self.sample_rate;
        let channels = self.channels;

        // Drop the stream to stop recording
        drop(self.stream);

        // Finalize the WAV file
        {
            let mut writer_guard = self.writer.lock().unwrap();
            if let Some(writer) = writer_guard.take() {
                writer.finalize()?;
            }
        }

        // Wait a moment for file to be fully written
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get file size
        let file_size_mb = if filepath.exists() {
            let size = std::fs::metadata(&filepath)?.len();
            (size as f64) / (1024.0 * 1024.0)
        } else {
            0.0
        };

        // Write completion status
        let status = serde_json::json!({
            "status": "completed",
            "session_id": session_id,
            "filename": filepath.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            "duration": duration.unwrap_or(0),
            "file_size_mb": format!("{:.2}", file_size_mb),
            "quality": "professional",
            "quality_info": RecordingQuality::professional(),
            "device": device_name,
            "sample_rate": sample_rate,
            "channels": channels,
            "frames_captured": frames_captured,
            "has_audio": has_audio,
        });

        std::fs::write(&status_file, serde_json::to_string_pretty(&status)?)?;
        tracing::info!("Recording completed: {:?}", filepath);

        Ok(filepath)
    }
}

/// Convert WAV file to M4A using FFmpeg with hardware acceleration and optimizations
pub async fn convert_wav_to_m4a(wav_path: &PathBuf, m4a_path: &PathBuf) -> Result<()> {
    crate::ffmpeg_encoder::convert_wav_to_m4a_optimized(wav_path, m4a_path).await
}

/// Merge two audio streams (loopback and microphone) into a single stereo file
/// Uses FFmpeg to handle sample rate mismatches and audio synchronization
/// Output format: Dual-mono stereo (Left=system audio, Right=microphone)
/// Supports direct M4A encoding (merge + encode in one pass for 50-70% faster processing)
pub async fn merge_audio_streams_smart(
    loopback_wav: &PathBuf,
    mic_wav: &PathBuf,
    output_path: &PathBuf,
    loopback_has_audio: bool,
    mic_has_audio: bool,
    quality: &RecordingQuality,
    output_format: crate::domain::AudioFormat,
    session_id: Option<&str>,
    observer: Option<std::sync::Arc<crate::status::JsonFileObserver>>,
    total_steps: u8,
) -> Result<()> {
    use std::process::Stdio;
    use std::time::Instant;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let format_str = match output_format {
        crate::domain::AudioFormat::Wav => "WAV",
        crate::domain::AudioFormat::M4a => "M4A (AAC)",
    };

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("🎧 AUDIO MERGE PROCESS STARTED");
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("📊 Merge Configuration:");
    tracing::info!(
        "    • Loopback (System Audio): {}",
        if loopback_has_audio { "✓ Present" } else { "✗ Silent" }
    );
    tracing::info!(
        "    • Microphone (User Audio):  {}",
        if mic_has_audio { "✓ Present" } else { "✗ Silent" }
    );
    tracing::info!("    • Output Format:          {}", format_str);
    tracing::info!("    • Output Sample Rate:     {} Hz", quality.sample_rate);
    tracing::info!("    • Output Channels:        {} (Stereo)", quality.channels);

    // Check if FFmpeg is available (cached on first run)
    use std::sync::OnceLock;
    static FFMPEG_AVAILABLE: OnceLock<bool> = OnceLock::new();

    let ffmpeg_available = FFMPEG_AVAILABLE.get_or_init(|| {
        // Perform synchronous FFmpeg availability check
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-version");

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW - prevents 8-second console creation delay
        }

        let check_result = cmd.output();
        check_result.is_ok()
    });

    if !ffmpeg_available {
        anyhow::bail!("FFmpeg is not installed or not in PATH. Please install FFmpeg for dual-channel recording.");
    }

    let target_sample_rate = quality.sample_rate.to_string();

    // Helper function to create FFmpeg command with hidden console on Windows
    #[cfg(windows)]
    fn setup_ffmpeg_command() -> Command {
        let mut cmd = Command::new("ffmpeg");
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd
    }

    #[cfg(not(windows))]
    fn setup_ffmpeg_command() -> Command {
        Command::new("ffmpeg")
    }

    // Helper function to add encoding parameters to FFmpeg command
    fn add_encoding_params(
        cmd: &mut Command,
        output_format: crate::domain::AudioFormat,
    ) {
        match output_format {
            crate::domain::AudioFormat::M4a => {
                // Optimized software AAC encoding (20-50x real-time performance)
                cmd.arg("-c:a").arg("aac");
                cmd.arg("-b:a").arg("192k"); // Explicit bitrate for consistent quality
                cmd.arg("-movflags").arg("faststart"); // Streaming-friendly
                cmd.arg("-threads").arg("auto"); // Use all CPU cores for encoding
            }
            crate::domain::AudioFormat::Wav => {
                // WAV output - no encoding needed, PCM passthrough
            }
        }
    }

    // Determine merge strategy based on audio detection flags
    tracing::info!("───────────────────────────────────────────────────────────");
    tracing::info!("⏳ Starting merge operation...");

    let start_time = Instant::now();

    // Emit interim status: preparing merge
    if let (Some(session_id), Some(observer)) = (session_id, observer.as_ref()) {
        tracing::info!("📝 Stage 2/{}: Preparing merge (detecting durations)...", total_steps);
        let _ = observer.write_processing_status_v2(
            session_id,
            "Preparing merge operation...",
            Some(2),
            Some(total_steps),
            Some("merging"),
            None,
            None,
        );
    }

    // Get audio duration from BOTH input files and use the maximum
    // On Windows, we merge two files (loopback + mic) which may have different durations
    // Run duration detection in parallel to reduce setup overhead (50% faster)
    let (loopback_duration_ms, mic_duration_ms) = tokio::join!(
        async { crate::ffmpeg_encoder::get_audio_duration_ms(loopback_wav).await.unwrap_or(0) },
        async { crate::ffmpeg_encoder::get_audio_duration_ms(mic_wav).await.unwrap_or(0) }
    );

    // Use the longer duration to ensure we don't show 100% prematurely
    let audio_duration_ms = std::cmp::max(loopback_duration_ms, mic_duration_ms);

    tracing::info!("📊 Duration detection:");
    tracing::info!("  ├─ Loopback: {} ms", loopback_duration_ms);
    tracing::info!("  ├─ Microphone: {} ms", mic_duration_ms);
    tracing::info!("  └─ Using maximum: {} ms", audio_duration_ms);

    // ALWAYS enable progress monitoring when we have a session and observer
    let enable_progress = session_id.is_some() && observer.is_some();

    // If duration detection failed for both, estimate from the larger file
    // Professional quality: 48kHz stereo 16-bit = 192,000 bytes/second
    let effective_duration_ms = if audio_duration_ms > 0 {
        audio_duration_ms
    } else {
        tracing::warn!("⚠ Duration detection failed for both files, estimating from larger file size");
        let loopback_size = std::fs::metadata(loopback_wav).map(|m| m.len()).unwrap_or(0);
        let mic_size = std::fs::metadata(mic_wav).map(|m| m.len()).unwrap_or(0);
        let larger_size = std::cmp::max(loopback_size, mic_size);

        if larger_size > 0 {
            const BYTES_PER_SECOND: u64 = 192000;
            let estimated_secs = larger_size / BYTES_PER_SECOND;
            let estimated_ms = estimated_secs * 1000;
            // Be conservative: add 20% buffer to prevent premature 100%
            let buffered_ms = (estimated_ms as f64 * 1.2) as u64;
            tracing::warn!("  ├─ Loopback file: {} bytes", loopback_size);
            tracing::warn!("  ├─ Microphone file: {} bytes", mic_size);
            tracing::warn!("  ├─ Estimated duration: {} ms", estimated_ms);
            tracing::warn!("  └─ Buffered estimate (20% extra): {} ms", buffered_ms);
            buffered_ms
        } else {
            tracing::warn!("  └─ Files unavailable, using default 300 seconds");
            300_000 // Default fallback: 5 minutes
        }
    };

    tracing::info!("Duration detection result: {} ms", audio_duration_ms);
    tracing::info!("Progress monitoring: {} (using effective duration: {} ms)",
        if enable_progress { "ENABLED" } else { "DISABLED" },
        effective_duration_ms);

    // Clone observer for the closure to own
    let observer_for_closure = observer.clone();

    // Helper closure to execute FFmpeg with or without progress monitoring
    let execute_ffmpeg = |mut cmd: Command| async move {
        if enable_progress {
            let session_id = session_id.unwrap();
            let observer = observer_for_closure.as_ref().unwrap();

            // Add progress flag
            cmd.arg("-progress").arg("pipe:2");
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn()?;
            let stderr = child.stderr.take().unwrap();
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            // Parse progress in real-time
            let mut current_speed = None;
            let mut encoding_stage_emitted = false;

            while let Ok(Some(line)) = lines.next_line().await {
                if line.starts_with("out_time_ms=") {
                    if let Ok(time_ms) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                        // Use effective_duration_ms (never 0) for reliable progress calculation
                        let progress_pct = ((time_ms as f64 / effective_duration_ms as f64) * 100.0).min(100.0) as u8;

                        // For M4A format, emit encoding stage when we're past merging (at 30% progress)
                        if matches!(output_format, crate::domain::AudioFormat::M4a)
                            && !encoding_stage_emitted
                            && progress_pct >= 30 {
                            tracing::info!("📝 Stage 3/{}: Encoding to M4A", total_steps);
                            let _ = observer.write_processing_status_v2(
                                session_id,
                                "Converting to M4A format...",
                                Some(3),
                                Some(total_steps),
                                Some("encoding"),
                                None,
                                None,
                            );
                            encoding_stage_emitted = true;
                        }

                        let _ = observer.update_ffmpeg_progress(
                            session_id,
                            progress_pct,
                            current_speed.clone(),
                        );

                        tracing::debug!(
                            "FFmpeg merge progress: {}% ({}/{} ms, speed: {:?})",
                            progress_pct,
                            time_ms,
                            effective_duration_ms,
                            current_speed
                        );
                    }
                } else if line.starts_with("speed=") {
                    current_speed = line.split('=').nth(1).map(|s| s.to_string());
                }
            }

            child.wait_with_output().await
        } else {
            cmd.output().await
        }
    };

    let output = if loopback_has_audio && mic_has_audio {
        // Scenario A: Both have audio - Create dual-mono stereo (L=loopback, R=mic)
        // Convert mic mono to stereo first, then merge with amerge
        tracing::info!("📋 Merge Strategy: Dual-mono stereo (L=loopback, R=microphone)");
        let cmd_build_start = std::time::Instant::now();

        let mut cmd = setup_ffmpeg_command();
        cmd.arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-i").arg(loopback_wav)
            .arg("-i").arg(mic_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=stereo[left];[1:a]aformat=channel_layouts=mono,asplit=2[ml][mr];[left][ml][mr]amerge=inputs=3,pan=stereo|c0<c0+c2|c1<c1+c2[aout]")
            .arg("-filter_threads").arg("auto")  // Enable parallel filter processing
            .arg("-map").arg("[aout]")
            .arg("-ar").arg(&target_sample_rate);

        add_encoding_params(&mut cmd, output_format);

        cmd.arg("-y")
            .arg(output_path);

        let cmd_build_elapsed = cmd_build_start.elapsed();
        tracing::debug!("  [BOTTLENECK] FFmpeg command construction: {:.3}s", cmd_build_elapsed.as_secs_f64());

        // Update status before FFmpeg starts
        if let (Some(session_id), Some(observer)) = (session_id, observer.as_ref()) {
            tracing::info!("🔀 Starting FFmpeg merge and encode...");
            let _ = observer.write_processing_status_v2(
                session_id,
                "Combining audio streams...",
                Some(2),
                Some(total_steps),
                Some("merging"),
                None,
                None,
            );
        }

        let ffmpeg_start = std::time::Instant::now();
        let result = execute_ffmpeg(cmd).await?;
        let ffmpeg_elapsed = ffmpeg_start.elapsed();
        tracing::info!("  [BOTTLENECK] FFmpeg execution: {:.3}s ({:.2}x real-time)",
            ffmpeg_elapsed.as_secs_f64(),
            audio_duration_ms as f64 / 1000.0 / ffmpeg_elapsed.as_secs_f64()
        );
        result
    } else if loopback_has_audio && !mic_has_audio {
        // Scenario B: Loopback only - Convert to stereo (duplicate to both channels)
        tracing::info!("📋 Merge Strategy: Using loopback only (duplicate system audio to stereo)");
        let ffmpeg_start = std::time::Instant::now();

        let mut cmd = setup_ffmpeg_command();
        cmd.arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-i")
            .arg(loopback_wav)
            .arg("-ac").arg("2")  // Direct channel conversion to stereo (faster than filter_complex)
            .arg("-ar")
            .arg(&target_sample_rate);

        add_encoding_params(&mut cmd, output_format);

        cmd.arg("-y")
            .arg(output_path);

        let result = execute_ffmpeg(cmd).await?;
        let ffmpeg_elapsed = ffmpeg_start.elapsed();
        tracing::info!("  [BOTTLENECK] FFmpeg execution: {:.3}s ({:.2}x real-time)",
            ffmpeg_elapsed.as_secs_f64(),
            audio_duration_ms as f64 / 1000.0 / ffmpeg_elapsed.as_secs_f64()
        );
        result
    } else if !loopback_has_audio && mic_has_audio {
        // Scenario C: Mic only - Convert mono to stereo (duplicate to both channels)
        tracing::info!("📋 Merge Strategy: Using microphone only (duplicate user audio to stereo)");
        let ffmpeg_start = std::time::Instant::now();

        let mut cmd = setup_ffmpeg_command();
        cmd.arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-i").arg(mic_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=mono,asplit=2[l][r];[l][r]amerge=inputs=2,pan=stereo|c0=c0|c1=c1[aout]")
            .arg("-filter_threads").arg("auto")  // Enable parallel filter processing
            .arg("-map").arg("[aout]")
            .arg("-ar").arg(&target_sample_rate);

        add_encoding_params(&mut cmd, output_format);

        cmd.arg("-y")
            .arg(output_path);

        let result = execute_ffmpeg(cmd).await?;
        let ffmpeg_elapsed = ffmpeg_start.elapsed();
        tracing::info!("  [BOTTLENECK] FFmpeg execution: {:.3}s ({:.2}x real-time)",
            ffmpeg_elapsed.as_secs_f64(),
            audio_duration_ms as f64 / 1000.0 / ffmpeg_elapsed.as_secs_f64()
        );
        result
    } else {
        // Scenario D: Neither has audio - Use loopback file (valid silent stereo)
        tracing::info!("📋 Merge Strategy: Both channels silent (creating silent stereo file)");
        let ffmpeg_start = std::time::Instant::now();

        let mut cmd = setup_ffmpeg_command();
        cmd.arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-i")
            .arg(loopback_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=stereo[aout]")
            .arg("-map")
            .arg("[aout]")
            .arg("-ar")
            .arg(&target_sample_rate);

        add_encoding_params(&mut cmd, output_format);

        cmd.arg("-y")
            .arg(output_path);

        let result = execute_ffmpeg(cmd).await?;
        let ffmpeg_elapsed = ffmpeg_start.elapsed();
        tracing::info!("  [BOTTLENECK] FFmpeg execution: {:.3}s ({:.2}x real-time)",
            ffmpeg_elapsed.as_secs_f64(),
            audio_duration_ms as f64 / 1000.0 / ffmpeg_elapsed.as_secs_f64()
        );
        result
    };

    let elapsed = start_time.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg merge failed: {}", stderr);
    }

    // Mark FFmpeg merge/encoding as complete to ensure UI transitions properly
    if let Some(session_id) = session_id {
        if let Some(observer) = &observer {
            // Emit finalizing stage
            let final_step = if matches!(output_format, crate::domain::AudioFormat::M4a) {
                4
            } else {
                3
            };
            tracing::info!("📝 Stage {}/{}: Finalizing", final_step, total_steps);
            let _ = observer.write_processing_status_v2(
                session_id,
                "Saving recording...",
                Some(final_step),
                Some(total_steps),
                Some("finalizing"),
                None,
                None,
            );

            // Small delay to show finalization stage
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            // Mark as complete
            let _ = observer.mark_ffmpeg_complete(session_id);
        }
    }

    // Log FFmpeg output for debugging
    let ffmpeg_stdout = String::from_utf8_lossy(&output.stdout);
    let ffmpeg_stderr = String::from_utf8_lossy(&output.stderr);
    if !ffmpeg_stdout.is_empty() {
        tracing::debug!("FFmpeg stdout: {}", ffmpeg_stdout);
    }
    if !ffmpeg_stderr.is_empty() {
        tracing::debug!("FFmpeg stderr: {}", ffmpeg_stderr);
    }

    // Get output file information
    let output_metadata = std::fs::metadata(output_path)?;
    let output_size_mb = output_metadata.len() as f64 / (1024.0 * 1024.0);

    // Bottleneck Analysis
    let total_elapsed = elapsed.as_secs_f64();
    let audio_duration_secs = audio_duration_ms as f64 / 1000.0;
    let processing_speed = if audio_duration_secs > 0.0 {
        audio_duration_secs / total_elapsed
    } else {
        0.0
    };

    tracing::info!("───────────────────────────────────────────────────────────");
    tracing::info!("✓ AUDIO MERGE COMPLETED SUCCESSFULLY");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Bottleneck Performance Metrics
    tracing::info!("🔍 BOTTLENECK ANALYSIS:");
    tracing::info!("    • Audio duration:      {:.2}s", audio_duration_secs);
    tracing::info!("    • Total time:          {:.2}s", total_elapsed);
    tracing::info!("    • Processing speed:   {:.2}x real-time", processing_speed);

    if processing_speed > 0.0 {
        if processing_speed < 1.0 {
            tracing::warn!("    ⚠️  SLOW: Processing slower than real-time ({:.2}x). Possible bottleneck:", processing_speed);
            tracing::warn!("         - Disk I/O bottleneck (slow drive or many read/writes)");
            tracing::warn!("         - Complex FFmpeg filter chain (amerge, pan filters)");
            tracing::warn!("         - CPU/Memory constraints");
        } else if processing_speed < 5.0 {
            tracing::info!("    ✓  ACCEPTABLE: Processing at {:.2}x real-time", processing_speed);
        } else {
            tracing::info!("    ✓✓ OPTIMAL: Processing at {:.2}x real-time (excellent performance)", processing_speed);
        }
    }

    tracing::info!("📊 Merge Results:");
    tracing::info!("    • Output file:     {:?}", output_path.file_name().unwrap_or_default());
    tracing::info!("    • Output format:   {}", format_str);
    tracing::info!("    • Output size:     {:.2} MB", output_size_mb);
    tracing::info!("    • Time elapsed:    {:.2}s", elapsed.as_secs_f64());
    tracing::info!("    • Sample rate:     {} Hz", quality.sample_rate);

    if matches!(output_format, crate::domain::AudioFormat::M4a) {
        tracing::info!("    • Encoder:         AAC (software, multi-threaded)");
        tracing::info!("    • Bitrate:         192 kbps");
    }

    tracing::info!("═══════════════════════════════════════════════════════════");
    Ok(())
}
