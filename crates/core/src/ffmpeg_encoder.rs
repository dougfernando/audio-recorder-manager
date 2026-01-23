use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

// Audio encoding constants (must match recorder.rs)
// Professional quality: 48kHz stereo 16-bit = 48000 * 2 channels * 2 bytes = 192,000 bytes/sec
const PROFESSIONAL_BYTES_PER_SECOND: u64 = 192_000;
const MIN_FILE_SIZE_FOR_ESTIMATION: u64 = 1000;

// Gemini file size limit (100MB)
pub const GEMINI_FILE_SIZE_LIMIT: u64 = 100 * 1024 * 1024;

/// Compression quality levels for reducing file size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionQuality {
    /// Standard quality: 44.1kHz stereo, 128kbps - good balance
    Standard,
    /// Quick/Low quality: 16kHz mono, 64kbps - maximum compression
    Quick,
}

impl CompressionQuality {
    /// Get the sample rate for this quality level
    pub fn sample_rate(&self) -> u32 {
        match self {
            CompressionQuality::Standard => 44100,
            CompressionQuality::Quick => 16000,
        }
    }

    /// Get the number of channels for this quality level
    pub fn channels(&self) -> u8 {
        match self {
            CompressionQuality::Standard => 2,
            CompressionQuality::Quick => 1,
        }
    }

    /// Get the bitrate for this quality level (in kbps)
    pub fn bitrate_kbps(&self) -> u32 {
        match self {
            CompressionQuality::Standard => 128,
            CompressionQuality::Quick => 64,
        }
    }

    /// Estimate the file size after compression given the original duration in milliseconds
    pub fn estimate_compressed_size(&self, duration_ms: u64) -> u64 {
        // Calculate size based on bitrate
        // Size = (bitrate in bits/sec * duration in seconds) / 8 (to get bytes)
        let duration_secs = duration_ms as f64 / 1000.0;
        let bitrate_bps = self.bitrate_kbps() as f64 * 1000.0;
        let size_bytes = (bitrate_bps * duration_secs) / 8.0;

        // Add 10% overhead for container format
        (size_bytes * 1.1) as u64
    }

    /// Get a human-readable description of this quality level
    pub fn description(&self) -> &'static str {
        match self {
            CompressionQuality::Standard => "Standard (44.1kHz stereo, 128kbps)",
            CompressionQuality::Quick => "Quick (16kHz mono, 64kbps)",
        }
    }
}

/// Information about compression options for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionEstimate {
    /// Original file size in bytes
    pub original_size: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether compression to Standard quality would bring it under the limit
    pub standard_would_fit: bool,
    /// Estimated size after Standard compression
    pub standard_estimated_size: u64,
    /// Whether compression to Quick quality would bring it under the limit
    pub quick_would_fit: bool,
    /// Estimated size after Quick compression
    pub quick_estimated_size: u64,
}

/// Estimate compression results for a file
pub async fn estimate_compression(audio_path: &PathBuf) -> Result<CompressionEstimate> {
    let metadata = std::fs::metadata(audio_path)?;
    let original_size = metadata.len();

    let duration_ms = get_audio_duration_ms(audio_path).await?;

    let standard_estimated = CompressionQuality::Standard.estimate_compressed_size(duration_ms);
    let quick_estimated = CompressionQuality::Quick.estimate_compressed_size(duration_ms);

    Ok(CompressionEstimate {
        original_size,
        duration_ms,
        standard_would_fit: standard_estimated < GEMINI_FILE_SIZE_LIMIT,
        standard_estimated_size: standard_estimated,
        quick_would_fit: quick_estimated < GEMINI_FILE_SIZE_LIMIT,
        quick_estimated_size: quick_estimated,
    })
}

/// Compress an audio file for transcription
/// Returns the path to the compressed file
pub async fn compress_audio_for_transcription(
    input_path: &PathBuf,
    quality: CompressionQuality,
    session_id: Option<&str>,
    observer: Option<Arc<crate::status::JsonFileObserver>>,
) -> Result<PathBuf> {
    use std::process::Stdio;
    use std::time::Instant;

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("🔧 AUDIO COMPRESSION FOR TRANSCRIPTION");
    tracing::info!("═══════════════════════════════════════════════════════════");

    let input_metadata = std::fs::metadata(input_path)?;
    let input_size_mb = input_metadata.len() as f64 / (1024.0 * 1024.0);

    tracing::info!("📂 Input file:  {:?}", input_path.file_name().unwrap_or_default());
    tracing::info!("📊 Input size:  {:.2} MB", input_size_mb);
    tracing::info!("🎚️  Target quality: {}", quality.description());

    // Generate output filename (add _compressed suffix before extension)
    let file_stem = input_path.file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?
        .to_string_lossy();
    let output_filename = format!("{}_compressed.m4a", file_stem);
    let output_path = input_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory"))?
        .join(&output_filename);

    tracing::info!("📂 Output file: {}", output_filename);

    // Get audio duration for progress calculation
    let audio_duration_ms = get_audio_duration_ms(input_path).await.unwrap_or(0);

    // Build FFmpeg command
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-loglevel").arg("error")
        .arg("-i").arg(input_path)
        .arg("-c:a").arg("aac")
        .arg("-b:a").arg(format!("{}k", quality.bitrate_kbps()))
        .arg("-ar").arg(quality.sample_rate().to_string())
        .arg("-ac").arg(quality.channels().to_string())
        .arg("-movflags").arg("faststart")
        .arg("-threads").arg("auto")
        .arg("-y")
        .arg(&output_path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    tracing::info!("───────────────────────────────────────────────────────────");
    tracing::info!("⏳ Starting compression...");

    let start_time = Instant::now();

    // If we have progress monitoring enabled, use it
    let output = if session_id.is_some() && observer.is_some() && audio_duration_ms > 0 {
        let session_id = session_id.unwrap();
        let observer = observer.as_ref().unwrap();

        // Add progress flag
        cmd.arg("-progress").arg("pipe:2");
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        let mut current_speed: Option<String> = None;
        let mut current_speed_float: f64 = 0.0;
        let encoding_start = std::time::Instant::now();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.starts_with("out_time_ms=") {
                if let Ok(time_us) = line.split('=').nth(1).unwrap_or("0").parse::<u64>() {
                    let time_ms = time_us / 1000;

                    let progress_pct = ((time_ms as f64 / audio_duration_ms as f64) * 100.0).min(100.0) as u8;

                    let remaining_audio_ms = audio_duration_ms.saturating_sub(time_ms);
                    let estimated_remaining_secs = if current_speed_float > 0.0 {
                        (remaining_audio_ms as f64 / 1000.0 / current_speed_float) as u64
                    } else {
                        let elapsed = encoding_start.elapsed().as_secs_f64();
                        if elapsed > 0.0 && time_ms > 0 {
                            let actual_speed = time_ms as f64 / 1000.0 / elapsed;
                            (remaining_audio_ms as f64 / 1000.0 / actual_speed) as u64
                        } else {
                            remaining_audio_ms / 1000 / 5
                        }
                    };

                    let _ = observer.write_compression_progress(
                        session_id,
                        progress_pct,
                        current_speed.clone(),
                        Some(audio_duration_ms),
                        Some(time_ms),
                        Some(estimated_remaining_secs),
                    );

                    tracing::debug!(
                        "Compression progress: {}% ({}/{} ms)",
                        progress_pct,
                        time_ms,
                        audio_duration_ms
                    );
                }
            } else if line.starts_with("speed=") {
                current_speed = line.split('=').nth(1).map(|s| s.to_string());
                current_speed_float = current_speed.as_ref()
                    .and_then(|s| s.trim_end_matches('x').parse().ok())
                    .unwrap_or(0.0);
            }
        }

        child.wait_with_output().await?
    } else {
        cmd.output().await?
    };

    let elapsed = start_time.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg compression failed: {}", stderr);
    }

    // Verify output file
    let output_metadata = std::fs::metadata(&output_path)?;
    let output_size_mb = output_metadata.len() as f64 / (1024.0 * 1024.0);
    let compression_ratio = (1.0 - (output_metadata.len() as f64 / input_metadata.len() as f64)) * 100.0;

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("✓ COMPRESSION COMPLETED SUCCESSFULLY");
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("📊 Results:");
    tracing::info!("    • Input size:      {:.2} MB", input_size_mb);
    tracing::info!("    • Output size:     {:.2} MB", output_size_mb);
    tracing::info!("    • Compression:     {:.1}%", compression_ratio);
    tracing::info!("    • Time elapsed:    {:.2}s", elapsed.as_secs_f64());

    // Verify output is under the limit
    if output_metadata.len() >= GEMINI_FILE_SIZE_LIMIT {
        tracing::warn!("⚠️ Compressed file is still over 100MB limit!");
    } else {
        tracing::info!("✓ Compressed file is under 100MB limit");
    }

    tracing::info!("═══════════════════════════════════════════════════════════");

    Ok(output_path)
}

/// FFmpeg progress information parsed from progress output
#[derive(Debug, Clone, Default)]
pub struct FfmpegProgress {
    pub out_time_ms: u64,
    pub speed: Option<String>,
    pub frame: Option<u64>,
}

/// Parse FFmpeg progress line (e.g., "out_time_ms=6000000" or "speed=12.0x")
fn parse_progress_line(line: &str) -> Option<(&str, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1].to_string()))
    } else {
        None
    }
}

/// Get audio duration in milliseconds using multiple fallback methods
/// This function ALWAYS returns a valid value (never fails completely)
pub async fn get_audio_duration_ms(audio_path: &PathBuf) -> Result<u64> {
    tracing::info!("🔍 Attempting duration detection for: {:?}", audio_path);

    // OPTIMIZATION: For WAV files, try header parsing FIRST (instant vs 7-9s for FFprobe)
    let is_wav_file = audio_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("wav"))
        .unwrap_or(false);

    if is_wav_file {
        tracing::debug!("  ├─ Method 1 (Priority): WAV header parsing...");
        if let Ok(duration_ms) = try_wav_header_duration(audio_path) {
            if duration_ms > 0 {
                tracing::info!("  ✓ Duration from WAV header: {} ms ({:.2}s) - INSTANT", duration_ms, duration_ms as f64 / 1000.0);
                return Ok(duration_ms);
            }
        }
        tracing::debug!("  ├─ WAV header parsing failed, falling back to FFprobe");
    }

    // Method 1/2: FFprobe format duration (fallback for non-WAV or failed WAV parsing)
    tracing::debug!("  ├─ Method {}: FFprobe format duration...", if is_wav_file { "2" } else { "1" });
    if let Ok(duration_ms) = try_ffprobe_format_duration(audio_path).await {
        if duration_ms > 0 {
            tracing::info!("  ✓ Duration from FFprobe format: {} ms ({:.2}s)", duration_ms, duration_ms as f64 / 1000.0);
            return Ok(duration_ms);
        }
    }

    // Method 2/3: FFprobe stream duration (alternative parser)
    tracing::debug!("  ├─ Method {}: FFprobe stream duration...", if is_wav_file { "3" } else { "2" });
    if let Ok(duration_ms) = try_ffprobe_stream_duration(audio_path).await {
        if duration_ms > 0 {
            tracing::info!("  ✓ Duration from FFprobe stream: {} ms ({:.2}s)", duration_ms, duration_ms as f64 / 1000.0);
            return Ok(duration_ms);
        }
    }

    // Method 3/4: Parse WAV header directly (for non-WAV files or as additional fallback)
    if !is_wav_file {
        tracing::debug!("  ├─ Method 3: WAV header parsing...");
        if let Ok(duration_ms) = try_wav_header_duration(audio_path) {
            if duration_ms > 0 {
                tracing::info!("  ✓ Duration from WAV header: {} ms ({:.2}s)", duration_ms, duration_ms as f64 / 1000.0);
                return Ok(duration_ms);
            }
        }
    }

    // Method 4/5: Estimate from file size (last resort)
    tracing::debug!("  ├─ Method {}: File size estimation...", if is_wav_file { "4" } else { "3" });
    if let Ok(duration_ms) = estimate_duration_from_filesize(audio_path) {
        if duration_ms > 0 {
            tracing::warn!("  ✓ Duration estimated from file size: {} ms ({:.2}s) (ESTIMATE)", duration_ms, duration_ms as f64 / 1000.0);
            return Ok(duration_ms);
        }
    }

    // All methods failed - this should rarely happen
    tracing::error!("  ✗ All duration detection methods failed for {:?}", audio_path);
    Ok(0)
}

/// Try FFprobe format duration (original method)
async fn try_ffprobe_format_duration(audio_path: &PathBuf) -> Result<u64> {
    let mut cmd = Command::new("ffprobe");
    cmd.arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(audio_path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                let duration_str = String::from_utf8_lossy(&output.stdout);
                let trimmed = duration_str.trim();

                if !trimmed.is_empty() {
                    if let Ok(duration_secs) = trimmed.parse::<f64>() {
                        if duration_secs > 0.0 {
                            let duration_ms = (duration_secs * 1000.0) as u64;
                            return Ok(duration_ms);
                        }
                    }
                }
            }
            Err(anyhow::anyhow!("FFprobe format method failed"))
        }
        Err(e) => Err(e.into()),
    }
}

/// Try FFprobe stream duration (alternative parser)
async fn try_ffprobe_stream_duration(audio_path: &PathBuf) -> Result<u64> {
    let mut cmd = Command::new("ffprobe");
    cmd.arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("stream=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(audio_path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                let duration_str = String::from_utf8_lossy(&output.stdout);
                let trimmed = duration_str.trim();

                if !trimmed.is_empty() {
                    // Try to parse first non-empty line (skip NaN values)
                    for line in trimmed.lines() {
                        if let Ok(duration_secs) = line.trim().parse::<f64>() {
                            if duration_secs.is_finite() && duration_secs > 0.0 {
                                let duration_ms = (duration_secs * 1000.0) as u64;
                                return Ok(duration_ms);
                            }
                        }
                    }
                }
            }
            Err(anyhow::anyhow!("FFprobe stream method failed"))
        }
        Err(e) => Err(e.into()),
    }
}

/// Parse WAV file header directly to get duration
/// WAV format: RIFF header -> chunks (fmt, data, etc.)
fn try_wav_header_duration(path: &PathBuf) -> Result<u64> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = [0u8; 4];

    // Read RIFF header
    file.read_exact(&mut buf)?;
    if &buf != b"RIFF" {
        return Err(anyhow::anyhow!("Not a valid WAV file (no RIFF header)"));
    }

    // Skip file size (4 bytes)
    file.read_exact(&mut buf)?;

    // Read WAVE format
    file.read_exact(&mut buf)?;
    if &buf != b"WAVE" {
        return Err(anyhow::anyhow!("Not a valid WAV file (no WAVE format)"));
    }

    let mut byte_rate = 0u32;
    let mut data_size = 0u32;

    // Read chunks
    loop {
        // Read chunk ID
        match file.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => break, // EOF
        }

        // Read chunk size (little-endian)
        let mut size_bytes = [0u8; 4];
        if file.read_exact(&mut size_bytes).is_err() {
            break;
        }
        let chunk_size = u32::from_le_bytes(size_bytes) as usize;

        if &buf == b"fmt " {
            // Read fmt chunk (at least 16 bytes for basic info)
            let mut fmt_data = vec![0u8; chunk_size];
            file.read_exact(&mut fmt_data)?;

            if fmt_data.len() >= 12 {
                // Bytes 4-7: sample rate (not used - we use byte_rate for duration calculation)
                // Bytes 8-11: byte rate (little-endian) - this is what we need for duration
                byte_rate = u32::from_le_bytes([
                    fmt_data[8],
                    fmt_data[9],
                    fmt_data[10],
                    fmt_data[11],
                ]);
            }
        } else if &buf == b"data" {
            // Found data chunk
            data_size = chunk_size as u32;

            // If we have both byte_rate and data_size, calculate duration
            if byte_rate > 0 && data_size > 0 {
                let duration_secs = data_size as f64 / byte_rate as f64;
                let duration_ms = (duration_secs * 1000.0) as u64;
                return Ok(duration_ms);
            }
            // Continue reading in case we haven't found fmt yet (unusual but possible)
        } else {
            // Skip unknown chunk
            if file.seek(SeekFrom::Current(chunk_size as i64)).is_err() {
                break;
            }
        }
    }

    if byte_rate > 0 && data_size > 0 {
        let duration_secs = data_size as f64 / byte_rate as f64;
        let duration_ms = (duration_secs * 1000.0) as u64;
        Ok(duration_ms)
    } else {
        Err(anyhow::anyhow!("Could not extract duration from WAV header"))
    }
}

/// Estimate duration from file size
/// Assumes professional quality: 48kHz stereo 16-bit = 192,000 bytes/second
fn estimate_duration_from_filesize(path: &PathBuf) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();

    if file_size < MIN_FILE_SIZE_FOR_ESTIMATION {
        return Err(anyhow::anyhow!("File too small to estimate duration"));
    }

    let duration_secs = file_size / PROFESSIONAL_BYTES_PER_SECOND;
    let duration_ms = duration_secs * 1000;

    Ok(duration_ms)
}

/// Convert WAV file to M4A using optimized software AAC encoding
pub async fn convert_wav_to_m4a_optimized(
    wav_path: &PathBuf,
    m4a_path: &PathBuf,
) -> Result<()> {
    convert_wav_to_m4a_with_progress(wav_path, m4a_path, None, None).await
}

/// Convert WAV file to M4A with optional progress monitoring
pub async fn convert_wav_to_m4a_with_progress(
    wav_path: &PathBuf,
    m4a_path: &PathBuf,
    session_id: Option<&str>,
    observer: Option<Arc<crate::status::JsonFileObserver>>,
) -> Result<()> {
    use std::process::Stdio;
    use std::time::Instant;

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("🎵 M4A ENCODING PROCESS STARTED");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Get file information
    let wav_metadata = std::fs::metadata(wav_path)?;
    let wav_size_mb = wav_metadata.len() as f64 / (1024.0 * 1024.0);
    tracing::info!("📂 Input file:  {:?}", wav_path.file_name().unwrap_or_default());
    tracing::info!("📊 Input size:  {:.2} MB", wav_size_mb);
    tracing::info!("📂 Output file: {:?}", m4a_path.file_name().unwrap_or_default());

    // Check if FFmpeg is available
    tracing::info!("⏳ Checking FFmpeg availability...");
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
    tracing::info!("✓ FFmpeg found");

    // Build FFmpeg command with optimized software AAC encoding
    let mut cmd = Command::new("ffmpeg");

    // Suppress FFmpeg banner and stats for cleaner output
    cmd.arg("-hide_banner");
    cmd.arg("-loglevel").arg("error");

    cmd.arg("-i").arg(wav_path);

    // Software AAC encoding configuration
    tracing::info!("⚙️  Encoder Configuration:");
    tracing::info!("    • Codec: AAC (software, optimized for audio)");
    tracing::info!("    • Bitrate: 192 kbps");
    tracing::info!("    • Quality: High (VBR mode)");
    tracing::info!("    • Threading: Auto (all available CPU cores)");
    tracing::info!("    • Expected speed: 20-50x real-time");

    cmd.arg("-c:a").arg("aac");
    cmd.arg("-b:a").arg("192k");
    cmd.arg("-movflags").arg("faststart");
    cmd.arg("-threads").arg("auto");

    // Finalize output
    cmd.arg("-y") // Overwrite output file
        .arg(m4a_path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    tracing::info!("───────────────────────────────────────────────────────────");
    tracing::info!("⏳ Starting M4A encoding...");
    tracing::info!("   This may take several minutes depending on file size");
    tracing::info!("   and available hardware...");

    let start_time = Instant::now();

    // Get audio duration for progress calculation
    let audio_duration_ms = get_audio_duration_ms(wav_path).await.unwrap_or(0);

    // If we have progress monitoring enabled, use it
    let output = if session_id.is_some() && observer.is_some() && audio_duration_ms > 0 {
        let session_id = session_id.unwrap();
        let observer = observer.as_ref().unwrap();

        // Add progress flag
        cmd.arg("-progress").arg("pipe:2");

        // Spawn with stderr piped
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        // Parse progress in real-time
        let mut current_progress = FfmpegProgress::default();
        let mut current_speed_float: f64 = 0.0;
        let encoding_start = std::time::Instant::now();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some((key, value)) = parse_progress_line(&line) {
                match key {
                    "out_time_ms" => {
                        if let Ok(time_ms) = value.parse::<u64>() {
                            current_progress.out_time_ms = time_ms;

                            // Calculate percentage
                            let progress_pct = if audio_duration_ms > 0 {
                                ((time_ms as f64 / audio_duration_ms as f64) * 100.0).min(100.0) as u8
                            } else {
                                0
                            };

                            // Calculate estimated remaining time
                            let remaining_audio_ms = audio_duration_ms.saturating_sub(time_ms);
                            let estimated_remaining_secs = if current_speed_float > 0.0 {
                                (remaining_audio_ms as f64 / 1000.0 / current_speed_float) as u64
                            } else {
                                let elapsed = encoding_start.elapsed().as_secs_f64();
                                if elapsed > 0.0 && time_ms > 0 {
                                    let actual_speed = time_ms as f64 / 1000.0 / elapsed;
                                    (remaining_audio_ms as f64 / 1000.0 / actual_speed) as u64
                                } else {
                                    remaining_audio_ms / 1000 / 5
                                }
                            };

                            // Update status file with enhanced progress info
                            let _ = observer.update_ffmpeg_progress(
                                session_id,
                                progress_pct,
                                current_progress.speed.clone(),
                                Some(audio_duration_ms),
                                Some(time_ms),
                                Some(estimated_remaining_secs),
                            );

                            tracing::debug!(
                                "FFmpeg progress: {}% ({}/{} ms) - Speed: {} - ETA: {}s",
                                progress_pct,
                                time_ms,
                                audio_duration_ms,
                                current_progress.speed.as_deref().unwrap_or("N/A"),
                                estimated_remaining_secs
                            );
                        }
                    }
                    "speed" => {
                        current_progress.speed = Some(value.clone());
                        // Parse speed as float for ETA calculation (e.g., "4.75x" -> 4.75)
                        current_speed_float = value.trim_end_matches('x').parse().unwrap_or(0.0);
                    }
                    _ => {}
                }
            }
        }

        child.wait_with_output().await?
    } else {
        // Fallback to standard execution without progress
        cmd.output().await?
    };

    let elapsed = start_time.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg conversion failed: {}", stderr);
    }

    // Mark FFmpeg encoding as complete to ensure UI transitions properly
    if let Some(session_id) = session_id {
        if let Some(observer) = &observer {
            let _ = observer.mark_ffmpeg_complete(session_id);
        }
    }

    // Get output file information
    let m4a_metadata = std::fs::metadata(m4a_path)?;
    let m4a_size_mb = m4a_metadata.len() as f64 / (1024.0 * 1024.0);
    let compression_ratio = (1.0 - (m4a_size_mb / wav_size_mb)) * 100.0;

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("✓ M4A ENCODING COMPLETED SUCCESSFULLY");
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("📊 Encoding Results:");
    tracing::info!("    • Input size:      {:.2} MB", wav_size_mb);
    tracing::info!("    • Output size:     {:.2} MB", m4a_size_mb);
    tracing::info!("    • Compression:     {:.1}%", compression_ratio);
    tracing::info!("    • Time elapsed:    {:.2}s", elapsed.as_secs_f64());
    tracing::info!("    • Speed (MB/min):  {:.2}", (wav_size_mb / elapsed.as_secs_f64()) * 60.0);

    tracing::info!("═══════════════════════════════════════════════════════════");

    Ok(())
}
