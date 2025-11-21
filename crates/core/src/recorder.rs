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
use std::sync::{Arc, Mutex};
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

/// Convert WAV file to M4A using FFmpeg
pub async fn convert_wav_to_m4a(wav_path: &PathBuf, m4a_path: &PathBuf) -> Result<()> {
    use tokio::process::Command;

    tracing::info!("Converting WAV to M4A: {:?} -> {:?}", wav_path, m4a_path);

    // Check if FFmpeg is available
    let mut ffmpeg_check = Command::new("ffmpeg");
    ffmpeg_check.arg("-version");

    #[cfg(windows)]
    ffmpeg_check.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let check_result = ffmpeg_check.output().await;

    if check_result.is_err() {
        anyhow::bail!(
            "FFmpeg is not installed or not in PATH. Please install FFmpeg to use M4A encoding."
        );
    }

    // Convert using FFmpeg with AAC codec
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(wav_path)
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("256k")
        .arg("-y") // Overwrite output file
        .arg(m4a_path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg conversion failed: {}", stderr);
    }

    tracing::info!("Successfully converted to M4A");
    Ok(())
}

/// Merge two audio streams (loopback and microphone) into a single stereo WAV file
/// Uses FFmpeg to handle sample rate mismatches and audio synchronization
/// Output format: Dual-mono stereo (Left=system audio, Right=microphone)
pub async fn merge_audio_streams_smart(
    loopback_wav: &PathBuf,
    mic_wav: &PathBuf,
    output_wav: &PathBuf,
    loopback_has_audio: bool,
    mic_has_audio: bool,
    quality: &RecordingQuality,
) -> Result<()> {
    use tokio::process::Command;

    tracing::info!(
        "Merging audio streams - Loopback: {}, Mic: {}",
        loopback_has_audio,
        mic_has_audio
    );

    // Check if FFmpeg is available
    let mut ffmpeg_check = Command::new("ffmpeg");
    ffmpeg_check.arg("-version");

    #[cfg(windows)]
    ffmpeg_check.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let check_result = ffmpeg_check.output().await;

    if check_result.is_err() {
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

    // Determine merge strategy based on audio detection flags
    let output = if loopback_has_audio && mic_has_audio {
        // Scenario A: Both have audio - Create dual-mono stereo (L=loopback, R=mic)
        // Convert mic mono to stereo first, then merge with amerge
        tracing::info!("Merging both channels (dual-mono stereo)");
        setup_ffmpeg_command()
            .arg("-i").arg(loopback_wav)
            .arg("-i").arg(mic_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=stereo[left];[1:a]aformat=channel_layouts=mono,asplit=2[ml][mr];[left][ml][mr]amerge=inputs=3,pan=stereo|c0<c0+c2|c1<c1+c2[aout]")
            .arg("-map").arg("[aout]")
            .arg("-ar").arg(&target_sample_rate)
            .arg("-y")
            .arg(output_wav)
            .output()
            .await?
    } else if loopback_has_audio && !mic_has_audio {
        // Scenario B: Loopback only - Convert to stereo (duplicate to both channels)
        tracing::info!("Using loopback only (microphone was silent)");
        setup_ffmpeg_command()
            .arg("-i")
            .arg(loopback_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=stereo[aout]")
            .arg("-map")
            .arg("[aout]")
            .arg("-ar")
            .arg(&target_sample_rate)
            .arg("-y")
            .arg(output_wav)
            .output()
            .await?
    } else if !loopback_has_audio && mic_has_audio {
        // Scenario C: Mic only - Convert mono to stereo (duplicate to both channels)
        tracing::info!("Using microphone only (system audio was silent)");
        setup_ffmpeg_command()
            .arg("-i").arg(mic_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=mono,asplit=2[l][r];[l][r]amerge=inputs=2,pan=stereo|c0=c0|c1=c1[aout]")
            .arg("-map").arg("[aout]")
            .arg("-ar").arg(&target_sample_rate)
            .arg("-y")
            .arg(output_wav)
            .output()
            .await?
    } else {
        // Scenario D: Neither has audio - Use loopback file (valid silent stereo)
        tracing::info!("Both channels were silent, creating silent stereo file");
        setup_ffmpeg_command()
            .arg("-i")
            .arg(loopback_wav)
            .arg("-filter_complex")
            .arg("[0:a]aformat=channel_layouts=stereo[aout]")
            .arg("-map")
            .arg("[aout]")
            .arg("-ar")
            .arg(&target_sample_rate)
            .arg("-y")
            .arg(output_wav)
            .output()
            .await?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg merge failed: {}", stderr);
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

    tracing::info!("Successfully merged audio streams");
    Ok(())
}
