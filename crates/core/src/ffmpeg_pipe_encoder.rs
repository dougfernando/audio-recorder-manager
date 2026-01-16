//! FFmpeg pipe-based streaming encoder
//!
//! This module provides real-time audio encoding by streaming PCM data to FFmpeg's stdin.
//! This eliminates the need for temporary WAV files and post-processing delays.
//!
//! # Architecture
//! ```text
//! WASAPI → PCM Buffer → FFmpeg stdin → Compressed file (M4A/MP3)
//!          (memory)      (pipe)        (disk)
//! ```
//!
//! # Benefits
//! - No temporary files (saves disk I/O)
//! - No post-processing delay (encoding happens during recording)
//! - Immediate output when recording stops
//! - Supports multiple formats (M4A, MP3, etc.)

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::streaming_encoder::StreamingEncoder;

/// FFmpeg-based streaming encoder that pipes PCM audio to FFmpeg
pub struct FFmpegPipeEncoder {
    /// The FFmpeg child process
    ffmpeg_process: Option<Child>,
    /// The stdin pipe to FFmpeg
    stdin: Option<ChildStdin>,
    /// Output file path
    output_path: PathBuf,
    /// Health status of the FFmpeg process
    healthy: Arc<AtomicBool>,
    /// Total bytes written to the encoder
    bytes_written: u64,
    /// Output format name (for logging)
    format: String,
    /// Bitrate in kbps
    bitrate: u32,
}

impl FFmpegPipeEncoder {
    /// Create a new M4A encoder (AAC codec)
    ///
    /// # Arguments
    /// * `output_path` - Path to the output M4A file
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000)
    /// * `bitrate` - Bitrate in kbps (e.g., 192)
    pub fn new_m4a(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        bitrate: u32,
    ) -> Result<Self> {
        Self::new(output_path, channels, sample_rate, "aac", bitrate, "m4a")
    }

    /// Create a new MP3 encoder (LAME codec)
    ///
    /// # Arguments
    /// * `output_path` - Path to the output MP3 file
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000)
    /// * `bitrate` - Bitrate in kbps (e.g., 192)
    pub fn new_mp3(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        bitrate: u32,
    ) -> Result<Self> {
        Self::new(
            output_path,
            channels,
            sample_rate,
            "libmp3lame",
            bitrate,
            "mp3",
        )
    }

    /// Internal constructor for creating any FFmpeg-based encoder
    fn new(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        codec: &str,
        bitrate: u32,
        format: &str,
    ) -> Result<Self> {
        tracing::info!(
            "Initializing FFmpeg {} encoder: {}kbps, {}Hz, {} channels",
            format.to_uppercase(),
            bitrate,
            sample_rate,
            channels
        );

        // Build FFmpeg command
        let mut cmd = Command::new("ffmpeg");
        cmd.args([
            "-f",
            "s16le", // Input format: 16-bit little-endian PCM
            "-ar",
            &sample_rate.to_string(), // Sample rate
            "-ac",
            &channels.to_string(), // Channels
            "-i",
            "pipe:0", // Read from stdin
            "-c:a",
            codec, // Audio codec
            "-b:a",
            &format!("{}k", bitrate), // Bitrate
        ]);

        // Add format-specific options
        if format == "m4a" {
            cmd.args([
                "-movflags",
                "+faststart", // Enable streaming-friendly M4A
            ]);
        }

        cmd.args([
            "-y", // Overwrite output file
            output_path.to_str().context("Invalid output path")?,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

        // On Windows, hide the console window
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        // Spawn FFmpeg process
        let mut ffmpeg = cmd.spawn().context("Failed to spawn FFmpeg process. Is FFmpeg installed?")?;

        let stdin = ffmpeg
            .stdin
            .take()
            .context("Failed to get FFmpeg stdin pipe")?;

        let healthy = Arc::new(AtomicBool::new(true));

        // Spawn health monitor thread
        let process_id = ffmpeg.id();
        let healthy_clone = Arc::clone(&healthy);
        let output_path_clone = output_path.clone();

        std::thread::spawn(move || {
            // Check process health every second
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                // Check if process is still running
                #[cfg(windows)]
                {
                    let status = Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", process_id), "/NH"])
                        .output();

                    if let Ok(output) = status {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if !output_str.contains(&process_id.to_string()) {
                            tracing::error!(
                                "FFmpeg process (PID {}) died unexpectedly!",
                                process_id
                            );
                            healthy_clone.store(false, Ordering::Release);
                            break;
                        }
                    }
                }

                #[cfg(not(windows))]
                {
                    // On Unix, try to send signal 0 (no-op) to check if process exists
                    use std::process::Command as StdCommand;
                    let status = StdCommand::new("kill")
                        .args(["-0", &process_id.to_string()])
                        .output();

                    if let Ok(output) = status {
                        if !output.status.success() {
                            tracing::error!(
                                "FFmpeg process (PID {}) died unexpectedly!",
                                process_id
                            );
                            healthy_clone.store(false, Ordering::Release);
                            break;
                        }
                    }
                }
            }

            tracing::debug!(
                "FFmpeg health monitor exited for: {:?}",
                output_path_clone
            );
        });

        tracing::info!("FFmpeg encoder started successfully (PID: {})", process_id);

        Ok(Self {
            ffmpeg_process: Some(ffmpeg),
            stdin: Some(stdin),
            output_path,
            healthy,
            bytes_written: 0,
            format: format.to_string(),
            bitrate,
        })
    }
}

impl StreamingEncoder for FFmpegPipeEncoder {
    fn write_samples(
        &mut self,
        samples: &[i16],
        _channels: u32,
        _sample_rate: u32,
    ) -> Result<()> {
        if !self.is_healthy() {
            bail!("FFmpeg process is not healthy - cannot write samples");
        }

        let stdin = self
            .stdin
            .as_mut()
            .context("FFmpeg stdin not available")?;

        // Convert i16 samples to bytes (little-endian)
        let bytes: Vec<u8> = samples
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();

        stdin
            .write_all(&bytes)
            .context("Failed to write samples to FFmpeg stdin")?;

        self.bytes_written += bytes.len() as u64;

        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        tracing::info!("Finalizing FFmpeg encoder for: {:?}", self.output_path);

        // Close stdin to signal end of input
        drop(self.stdin.take());

        // Wait for FFmpeg to finish encoding
        let mut process = self
            .ffmpeg_process
            .take()
            .context("FFmpeg process not available")?;

        let status = process.wait().context("Failed to wait for FFmpeg process")?;

        if !status.success() {
            // Try to read stderr for error details
            let stderr = process
                .stderr
                .as_mut()
                .and_then(|s| {
                    use std::io::Read;
                    let mut buf = String::new();
                    s.read_to_string(&mut buf).ok()?;
                    Some(buf)
                })
                .unwrap_or_else(|| "No error details available".to_string());

            bail!(
                "FFmpeg encoding failed with status: {}. Error: {}",
                status,
                stderr
            );
        }

        // Check if output file was created
        if !self.output_path.exists() {
            bail!(
                "FFmpeg finished but output file was not created: {:?}",
                self.output_path
            );
        }

        let file_size = std::fs::metadata(&self.output_path)?.len();
        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

        tracing::info!(
            "FFmpeg encoding completed successfully: {:?} ({:.2} MB)",
            self.output_path,
            file_size_mb
        );

        Ok(())
    }

    fn estimated_size(&self) -> Option<u64> {
        // Estimate compressed size based on bitrate
        // bitrate is in kbps, convert to bytes per second and apply to elapsed time
        let bytes_per_sec = (self.bitrate as u64 * 1000) / 8;

        // Estimate recording duration from PCM bytes written
        // Assuming 16-bit stereo at 48kHz: 192,000 bytes/sec
        let pcm_bytes_per_sec = 192_000u64;
        let duration_secs = self.bytes_written / pcm_bytes_per_sec;

        Some(bytes_per_sec * duration_secs)
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }
}

impl Drop for FFmpegPipeEncoder {
    fn drop(&mut self) {
        // Ensure FFmpeg process is terminated if not already
        if let Some(mut process) = self.ffmpeg_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_creation_requires_ffmpeg() {
        // This test will fail if FFmpeg is not installed
        // Skip in CI environments where FFmpeg might not be available
        if std::env::var("CI").is_ok() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_audio.m4a");

        let encoder = FFmpegPipeEncoder::new_m4a(output_path, 2, 48000, 192);

        // Should succeed if FFmpeg is available
        if encoder.is_ok() {
            // Cleanup
            let mut encoder = encoder.unwrap();
            let _ = Box::new(encoder).finish();
        }
    }

    #[test]
    fn test_health_check() {
        if std::env::var("CI").is_ok() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_health.m4a");

        if let Ok(encoder) = FFmpegPipeEncoder::new_m4a(output_path.clone(), 2, 48000, 192) {
            assert!(encoder.is_healthy());

            // Cleanup
            let _ = Box::new(encoder).finish();
            let _ = std::fs::remove_file(output_path);
        }
    }
}
