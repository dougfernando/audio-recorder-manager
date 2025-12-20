use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Global cache for hardware encoder detection results
/// Initialized once on first access and reused for the lifetime of the application
static ENCODER_CACHE: OnceLock<EncoderCapabilities> = OnceLock::new();

/// Persistent hardware encoder cache stored on disk
/// Cached encoders remain valid across app executions since hardware doesn't change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentEncoderCache {
    pub available_encoders: Vec<HardwareEncoder>,
    pub preferred_encoder: EncoderChoice,
    pub ffmpeg_version: String,
    pub cached_at: String,
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

    let mut _sample_rate = 0u32; // Kept for reference, not currently used in calculation
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
                // Bytes 4-7: sample rate (little-endian)
                _sample_rate = u32::from_le_bytes([
                    fmt_data[4],
                    fmt_data[5],
                    fmt_data[6],
                    fmt_data[7],
                ]);

                // Bytes 8-11: byte rate (little-endian)
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

    if file_size < 1000 {
        return Err(anyhow::anyhow!("File too small to estimate duration"));
    }

    // Professional quality: 48000 Hz * 2 channels * 2 bytes = 192000 bytes/sec
    const BYTES_PER_SECOND: u64 = 192000;
    let duration_secs = file_size / BYTES_PER_SECOND;
    let duration_ms = duration_secs * 1000;

    Ok(duration_ms)
}

/// Represents available hardware encoders on the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareEncoder {
    /// Intel Quick Sync (H.264/H.265)
    IntelQuickSync,
    /// NVIDIA NVENC (H.264/H.265)
    NvidiaEnc,
    /// AMD VCE (H.264/H.265)
    AmdVce,
}

/// Result of hardware encoder detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderCapabilities {
    pub available_encoders: Vec<HardwareEncoder>,
    pub preferred_encoder: EncoderChoice,
}

/// Final encoder choice to use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncoderChoice {
    Hardware(HardwareEncoder),
    Software,
}

/// Get the path to the persistent encoder cache file
fn get_encoder_cache_path() -> PathBuf {
    let config_dir = if let Ok(config_home) = std::env::var("APPDATA") {
        PathBuf::from(config_home)
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    config_dir
        .join("audio-recorder-manager")
        .join("encoder-cache.json")
}

/// Get the current FFmpeg version
async fn get_ffmpeg_version() -> String {
    match Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
    {
        Ok(output) => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Extract first line (contains version info)
            version_str.lines().next().unwrap_or("unknown").to_string()
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Load hardware encoder cache from disk if it exists and is valid
async fn load_persistent_encoder_cache() -> Option<EncoderCapabilities> {
    let cache_path = get_encoder_cache_path();

    if !cache_path.exists() {
        tracing::debug!("No persistent encoder cache found at {:?}", cache_path);
        return None;
    }

    match std::fs::read_to_string(&cache_path) {
        Ok(content) => {
            match serde_json::from_str::<PersistentEncoderCache>(&content) {
                Ok(cached) => {
                    let current_version = get_ffmpeg_version().await;

                    // Validate that FFmpeg version matches (encoders could change with version)
                    if cached.ffmpeg_version == current_version {
                        tracing::info!(
                            "✓ Loaded cached hardware encoders (cached at {}, version: {})",
                            cached.cached_at,
                            current_version
                        );
                        return Some(EncoderCapabilities {
                            available_encoders: cached.available_encoders,
                            preferred_encoder: cached.preferred_encoder,
                        });
                    } else {
                        tracing::info!(
                            "⚠️  Encoder cache is stale (FFmpeg version changed). Clearing cache and re-detecting."
                        );
                        let _ = std::fs::remove_file(&cache_path);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse encoder cache: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::debug!("Failed to read encoder cache: {}", e);
        }
    }

    None
}

/// Save hardware encoder detection results to persistent cache
async fn save_persistent_encoder_cache(capabilities: &EncoderCapabilities) -> Result<()> {
    let cache_path = get_encoder_cache_path();

    // Ensure the directory exists
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ffmpeg_version = get_ffmpeg_version().await;
    let cached = PersistentEncoderCache {
        available_encoders: capabilities.available_encoders.clone(),
        preferred_encoder: capabilities.preferred_encoder.clone(),
        ffmpeg_version,
        cached_at: chrono::Local::now().to_rfc3339(),
    };

    let json = serde_json::to_string_pretty(&cached)?;
    std::fs::write(&cache_path, json)?;

    tracing::debug!("✓ Saved encoder cache to {:?}", cache_path);
    Ok(())
}

/// Detect available hardware encoders on the system
pub async fn detect_hardware_encoders() -> Result<EncoderCapabilities> {
    let mut available_encoders = Vec::new();

    tracing::info!("🔍 Detecting available hardware encoders on system...");
    tracing::debug!("Running encoder detection in PARALLEL for QSV, NVENC, and AMF...");

    // Detect all three encoders in parallel (3x faster than sequential)
    // Each encoder detection typically takes ~1.6-10 seconds, so running in parallel
    // reduces the total time to the longest single check instead of the sum of all three
    let (has_qsv, has_nvenc, has_amf) = tokio::join!(
        detect_encoder("h264_qsv"),
        detect_encoder("h264_nvenc"),
        detect_encoder("h264_amf")
    );

    // Process results in order of preference
    if has_qsv {
        tracing::info!("✓ Intel Quick Sync (h264_qsv) detected - enables GPU acceleration");
        available_encoders.push(HardwareEncoder::IntelQuickSync);
    }

    if has_nvenc {
        tracing::info!("✓ NVIDIA NVENC (h264_nvenc) detected - enables GPU acceleration");
        available_encoders.push(HardwareEncoder::NvidiaEnc);
    }

    if has_amf {
        tracing::info!("✓ AMD VCE (h264_amf) detected - enables GPU acceleration");
        available_encoders.push(HardwareEncoder::AmdVce);
    }

    // Select preferred encoder (in order of preference)
    let preferred_encoder = if !available_encoders.is_empty() {
        let encoder_name = match available_encoders.first().unwrap() {
            HardwareEncoder::IntelQuickSync => "Intel Quick Sync",
            HardwareEncoder::NvidiaEnc => "NVIDIA NVENC",
            HardwareEncoder::AmdVce => "AMD VCE",
        };
        tracing::info!(
            "🚀 Using hardware encoder: {} (expects 3-5x faster encoding)",
            encoder_name
        );
        EncoderChoice::Hardware(available_encoders[0])
    } else {
        tracing::warn!("⚠️  No hardware encoders detected");
        tracing::info!("📊 Will use optimized software encoding (slower than hardware, but still optimized)");
        EncoderChoice::Software
    };

    Ok(EncoderCapabilities {
        available_encoders,
        preferred_encoder,
    })
}

/// Detect hardware encoders with multi-level caching
/// 1. In-memory cache (app lifetime) - <1ms
/// 2. Persistent disk cache (persists across app restarts) - <1ms
/// 3. Live detection (first time or version mismatch) - ~10s
pub async fn detect_hardware_encoders_cached() -> Result<EncoderCapabilities> {
    // Level 1: Try in-memory cache (app lifetime)
    if let Some(cached) = ENCODER_CACHE.get() {
        tracing::debug!("✓ Using in-memory cached hardware encoders");
        return Ok(cached.clone());
    }

    // Level 2: Try persistent disk cache
    if let Some(cached) = load_persistent_encoder_cache().await {
        // Store in memory cache for faster subsequent access
        let _ = ENCODER_CACHE.set(cached.clone());
        return Ok(cached);
    }

    // Level 3: Perform live detection (will take ~10 seconds with parallel detection)
    tracing::info!("🔍 Performing live hardware encoder detection (will cache for future use)...");
    let start = std::time::Instant::now();
    let capabilities = detect_hardware_encoders().await?;
    let elapsed = start.elapsed();

    tracing::info!(
        "✓ Detection completed in {:.2}s - saving to persistent cache",
        elapsed.as_secs_f64()
    );

    // Cache the result in memory
    let _ = ENCODER_CACHE.set(capabilities.clone());

    // Save to persistent disk cache
    if let Err(e) = save_persistent_encoder_cache(&capabilities).await {
        tracing::warn!(
            "Failed to save encoder cache to disk: {} (will use in-memory cache)",
            e
        );
    }

    Ok(capabilities)
}

/// Check if a specific encoder is available in FFmpeg
async fn detect_encoder(encoder_name: &str) -> bool {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-encoders").arg("-hide_banner");

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    match cmd.output().await {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // FFmpeg output shows available encoders with their names
            output_str.contains(encoder_name)
        }
        Err(_) => false,
    }
}

/// Convert WAV file to M4A using best available encoder with optimization flags
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
    // Note: Hardware encoders (h264_qsv, nvenc, amf) are for VIDEO only, not audio.
    // Audio encoding uses software AAC which is fast and efficient.
    let mut cmd = Command::new("ffmpeg");

    // Suppress FFmpeg banner and stats for cleaner output
    cmd.arg("-hide_banner");
    cmd.arg("-loglevel").arg("error");

    cmd.arg("-i").arg(wav_path);

    // Direct software AAC encoding (no hardware detection needed for audio)
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

                            // Update status file
                            let _ = observer.update_ffmpeg_progress(
                                session_id,
                                progress_pct,
                                current_progress.speed.clone(),
                            );

                            tracing::debug!(
                                "FFmpeg progress: {}% ({}/{} ms) - Speed: {}",
                                progress_pct,
                                time_ms,
                                audio_duration_ms,
                                current_progress.speed.as_deref().unwrap_or("N/A")
                            );
                        }
                    }
                    "speed" => {
                        current_progress.speed = Some(value);
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
    tracing::info!("    • Encoder used:    Software AAC (multi-threaded)");

    tracing::info!("═══════════════════════════════════════════════════════════");

    Ok(())
}
