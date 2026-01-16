# Implementation Plan: Direct Compressed Audio Recording

## Executive Summary

This plan outlines the implementation of real-time compressed audio recording to eliminate post-processing delays. The solution uses FFmpeg pipes to stream PCM audio directly to compressed formats (M4A, MP3, Opus) during recording.

---

## Architecture Overview

### Current Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ RECORDING PHASE                                             │
├─────────────────────────────────────────────────────────────┤
│ WASAPI Loopback → temp_loopback.wav (330MB for 30min)      │
│ WASAPI Mic      → temp_mic.wav (330MB for 30min)           │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ POST-PROCESSING PHASE (10-30 seconds delay)                │
├─────────────────────────────────────────────────────────────┤
│ FFmpeg: Merge channels → merged.wav                         │
│ FFmpeg: Encode → final.m4a (43MB for 30min)                │
│ Cleanup: Delete temp files                                  │
└─────────────────────────────────────────────────────────────┘
```

### New Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ RECORDING PHASE (No post-processing needed!)               │
├─────────────────────────────────────────────────────────────┤
│ WASAPI Loopback → FFmpeg pipe → loopback.m4a (43MB)        │
│ WASAPI Mic      → FFmpeg pipe → mic.m4a (43MB)             │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ OPTIONAL MERGE PHASE (5-10 seconds, only if needed)        │
├─────────────────────────────────────────────────────────────┤
│ FFmpeg: Fast remux (no re-encoding) → final.m4a            │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### Phase 1: Core Infrastructure (Priority: HIGH)

#### 1.1 Create Streaming Encoder Trait

**File**: `crates/core/src/streaming_encoder.rs` (new)

```rust
use anyhow::Result;
use std::path::PathBuf;

/// Trait for real-time audio encoding during recording
pub trait StreamingEncoder: Send {
    /// Write PCM samples to the encoder
    /// - samples: Interleaved PCM samples (i16)
    /// - channels: Number of audio channels (1=mono, 2=stereo)
    /// - sample_rate: Sample rate in Hz
    fn write_samples(&mut self, samples: &[i16], channels: u32, sample_rate: u32) -> Result<()>;

    /// Finish encoding and flush remaining data
    fn finish(self: Box<Self>) -> Result<()>;

    /// Get estimated output file size in bytes (if available)
    fn estimated_size(&self) -> Option<u64>;

    /// Check if encoder process is healthy
    fn is_healthy(&self) -> bool;
}

/// Format-specific encoder implementations
pub enum EncoderFormat {
    Wav,
    M4a { bitrate: u32 },      // AAC encoding
    Mp3 { bitrate: u32 },      // MP3 encoding
    Opus { bitrate: u32 },     // Opus encoding
}
```

#### 1.2 Implement FFmpeg Pipe Encoder

**File**: `crates/core/src/ffmpeg_pipe_encoder.rs` (new)

```rust
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio, ChildStdin};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::streaming_encoder::StreamingEncoder;

pub struct FFmpegPipeEncoder {
    ffmpeg_process: Child,
    stdin: ChildStdin,
    output_path: PathBuf,
    healthy: Arc<AtomicBool>,
    bytes_written: u64,
    format: String,
}

impl FFmpegPipeEncoder {
    pub fn new_m4a(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        bitrate: u32,
    ) -> Result<Self> {
        Self::new(output_path, channels, sample_rate, "aac", bitrate, "m4a")
    }

    pub fn new_mp3(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        bitrate: u32,
    ) -> Result<Self> {
        Self::new(output_path, channels, sample_rate, "libmp3lame", bitrate, "mp3")
    }

    fn new(
        output_path: PathBuf,
        channels: u32,
        sample_rate: u32,
        codec: &str,
        bitrate: u32,
        format: &str,
    ) -> Result<Self> {
        let mut ffmpeg = Command::new("ffmpeg")
            .args([
                "-f", "s16le",                          // 16-bit little-endian PCM
                "-ar", &sample_rate.to_string(),        // Sample rate
                "-ac", &channels.to_string(),           // Channels
                "-i", "pipe:0",                         // Read from stdin
                "-c:a", codec,                          // Audio codec
                "-b:a", &format!("{}k", bitrate),      // Bitrate
                "-movflags", "+faststart",              // Enable streaming (M4A)
                "-y",                                   // Overwrite output
                output_path.to_str().context("Invalid path")?,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn FFmpeg process")?;

        let stdin = ffmpeg
            .stdin
            .take()
            .context("Failed to get FFmpeg stdin")?;

        let healthy = Arc::new(AtomicBool::new(true));

        // Spawn health monitor thread
        let process_id = ffmpeg.id();
        let healthy_clone = Arc::clone(&healthy);
        std::thread::spawn(move || {
            // Periodically check if FFmpeg process is still running
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                // Check process status using platform-specific method
                #[cfg(windows)]
                {
                    use std::process::Command;
                    let status = Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", process_id)])
                        .output();
                    if let Ok(output) = status {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if !output_str.contains(&process_id.to_string()) {
                            healthy_clone.store(false, Ordering::Release);
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            ffmpeg_process: ffmpeg,
            stdin,
            output_path,
            healthy,
            bytes_written: 0,
            format: format.to_string(),
        })
    }
}

impl StreamingEncoder for FFmpegPipeEncoder {
    fn write_samples(&mut self, samples: &[i16], _channels: u32, _sample_rate: u32) -> Result<()> {
        if !self.is_healthy() {
            bail!("FFmpeg process is not healthy");
        }

        // Convert i16 samples to bytes (little-endian)
        let bytes = samples
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect::<Vec<u8>>();

        self.stdin
            .write_all(&bytes)
            .context("Failed to write to FFmpeg stdin")?;

        self.bytes_written += bytes.len() as u64;

        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        // Close stdin to signal end of input
        drop(self.stdin);

        // Wait for FFmpeg to finish
        let status = self
            .ffmpeg_process
            .wait()
            .context("Failed to wait for FFmpeg process")?;

        if !status.success() {
            bail!("FFmpeg encoding failed with status: {}", status);
        }

        Ok(())
    }

    fn estimated_size(&self) -> Option<u64> {
        // Rough estimate: compressed size is ~1/8 of PCM for 192kbps AAC
        Some(self.bytes_written / 8)
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }
}
```

#### 1.3 Implement WAV Encoder Wrapper

**File**: `crates/core/src/wav_encoder.rs` (new)

```rust
use anyhow::{Context, Result};
use hound::{WavSpec, WavWriter};
use std::io::BufWriter;
use std::fs::File;
use std::path::PathBuf;
use crate::streaming_encoder::StreamingEncoder;

pub struct WavStreamingEncoder {
    writer: WavWriter<BufWriter<File>>,
    spec: WavSpec,
}

impl WavStreamingEncoder {
    pub fn new(output_path: PathBuf, channels: u32, sample_rate: u32) -> Result<Self> {
        let spec = WavSpec {
            channels: channels as u16,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(output_path, spec)
            .context("Failed to create WAV writer")?;

        Ok(Self { writer, spec })
    }
}

impl StreamingEncoder for WavStreamingEncoder {
    fn write_samples(&mut self, samples: &[i16], _channels: u32, _sample_rate: u32) -> Result<()> {
        for &sample in samples {
            self.writer
                .write_sample(sample)
                .context("Failed to write WAV sample")?;
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<()> {
        self.writer
            .finalize()
            .context("Failed to finalize WAV file")?;
        Ok(())
    }

    fn estimated_size(&self) -> Option<u64> {
        None // WAV writer handles this internally
    }

    fn is_healthy(&self) -> bool {
        true // WAV writing doesn't involve external process
    }
}
```

#### 1.4 Modify WASAPI Recorders

**Files**:
- `crates/core/src/wasapi_loopback.rs` (modify)
- `crates/core/src/wasapi_microphone.rs` (modify)

**Changes**:
1. Replace `WavWriter` with `Box<dyn StreamingEncoder>`
2. Accept encoder as constructor parameter
3. Update `write_samples` calls to use trait method

**Example**:
```rust
pub struct WasapiLoopbackRecorder {
    is_recording: Arc<AtomicBool>,
    frames_captured: Arc<AtomicU64>,
    has_audio: Arc<AtomicBool>,
    sample_rate: u32,
}

impl WasapiLoopbackRecorder {
    pub fn new(encoder: Box<dyn StreamingEncoder>) -> Result<Self> {
        // ... existing code ...

        std::thread::spawn(move || {
            let _ = Self::recording_thread(
                encoder,  // Pass encoder instead of filepath
                is_recording_clone,
                frames_captured_clone,
                has_audio_clone,
            );
        });

        // ... rest of code ...
    }

    fn recording_thread(
        mut encoder: Box<dyn StreamingEncoder>,
        // ... other params ...
    ) -> Result<()> {
        // In capture loop, replace WAV writing with:
        encoder.write_samples(&samples, channels, sample_rate)?;
    }
}
```

---

### Phase 2: API Updates (Priority: HIGH)

#### 2.1 Update RecordingConfig

**File**: `crates/core/src/lib.rs` (modify)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub duration_seconds: Option<u32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub output_format: OutputFormat,  // NEW
    pub quality_preset: QualityPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Wav,
    M4a { bitrate: u32 },
    Mp3 { bitrate: u32 },
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            duration_seconds: None,
            sample_rate: 48000,
            channels: 2,
            output_format: OutputFormat::M4a { bitrate: 192 },  // NEW DEFAULT
            quality_preset: QualityPreset::Professional,
        }
    }
}
```

#### 2.2 Update start_recording Function

**File**: `crates/core/src/lib.rs` (modify)

```rust
pub async fn start_recording(
    config: RecordingConfig,
    should_merge: bool,
) -> Result<RecordingSession> {
    // Create encoders based on format
    let loopback_encoder = create_encoder(
        &config.output_format,
        loopback_path.clone(),
        config.channels as u32,
        config.sample_rate,
    )?;

    let mic_encoder = create_encoder(
        &config.output_format,
        mic_path.clone(),
        config.channels as u32,
        config.sample_rate,
    )?;

    // Start recording with encoders
    let loopback_recorder = WasapiLoopbackRecorder::new(loopback_encoder)?;
    let mic_recorder = WasapiMicrophoneRecorder::new(mic_encoder)?;

    // ... rest of recording logic ...
}

fn create_encoder(
    format: &OutputFormat,
    path: PathBuf,
    channels: u32,
    sample_rate: u32,
) -> Result<Box<dyn StreamingEncoder>> {
    match format {
        OutputFormat::Wav => {
            Ok(Box::new(WavStreamingEncoder::new(path, channels, sample_rate)?))
        }
        OutputFormat::M4a { bitrate } => {
            Ok(Box::new(FFmpegPipeEncoder::new_m4a(path, channels, sample_rate, *bitrate)?))
        }
        OutputFormat::Mp3 { bitrate } => {
            Ok(Box::new(FFmpegPipeEncoder::new_mp3(path, channels, sample_rate, *bitrate)?))
        }
    }
}
```

---

### Phase 3: CLI Updates (Priority: MEDIUM)

#### 3.1 Update CLI Command Parsing

**File**: `crates/cli/src/main.rs` (modify)

```rust
// Old: record <duration> <format>
// New: record <duration> <format> [bitrate]

// Examples:
// record 300 m4a         -> M4A at 192kbps (default)
// record 300 m4a 256     -> M4A at 256kbps
// record 300 mp3         -> MP3 at 192kbps
// record 300 wav         -> WAV (lossless)

match args[0].as_str() {
    "record" => {
        let duration = args[1].parse::<i32>()?;
        let format_str = args.get(2).map(|s| s.as_str()).unwrap_or("m4a");
        let bitrate = args.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(192);

        let output_format = match format_str {
            "wav" => OutputFormat::Wav,
            "m4a" => OutputFormat::M4a { bitrate },
            "mp3" => OutputFormat::Mp3 { bitrate },
            _ => bail!("Unknown format: {}", format_str),
        };

        let config = RecordingConfig {
            duration_seconds: if duration > 0 { Some(duration as u32) } else { None },
            output_format,
            ..Default::default()
        };

        start_recording(config, true).await?;
    }
}
```

---

### Phase 4: Post-Processing Optimization (Priority: LOW)

#### 4.1 Fast Remux for Compressed Files

When merging two M4A files (loopback + mic), use FFmpeg remux instead of re-encoding:

```rust
pub async fn fast_merge_compressed(
    loopback_path: &PathBuf,
    mic_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<()> {
    // Use FFmpeg filter_complex with amerge + pan for channel mapping
    // No re-encoding, just remux (very fast ~2-3 seconds for 30min audio)

    let status = Command::new("ffmpeg")
        .args([
            "-i", loopback_path.to_str().unwrap(),
            "-i", mic_path.to_str().unwrap(),
            "-filter_complex",
            "[0:a][1:a]amerge=inputs=2,pan=stereo|c0=c0|c1=c1[out]",
            "-map", "[out]",
            "-c:a", "copy",  // No re-encoding!
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        bail!("Fast merge failed");
    }

    Ok(())
}
```

---

## File Structure Changes

### New Files
```
crates/core/src/
├── streaming_encoder.rs          (NEW - trait definition)
├── ffmpeg_pipe_encoder.rs        (NEW - FFmpeg pipe implementation)
├── wav_encoder.rs                (NEW - WAV wrapper)
└── encoders/                     (NEW - future encoders)
    └── opus_encoder.rs           (FUTURE - native Opus)
```

### Modified Files
```
crates/core/src/
├── lib.rs                        (MODIFY - add OutputFormat enum, update API)
├── wasapi_loopback.rs            (MODIFY - use StreamingEncoder)
├── wasapi_microphone.rs          (MODIFY - use StreamingEncoder)
└── Cargo.toml                    (MODIFY - no new deps for Phase 1)

crates/cli/src/
└── main.rs                       (MODIFY - update command parsing)
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_encoder_creation() {
        // Test encoder initialization
    }

    #[test]
    fn test_wav_encoder_compatibility() {
        // Ensure WAV encoder produces same output as before
    }

    #[test]
    fn test_encoder_health_check() {
        // Test health monitoring
    }
}
```

### Integration Tests
1. Record 10-second test audio in all formats
2. Verify file sizes are reasonable
3. Check audio playback
4. Verify no data corruption
5. Test encoder failure recovery

### Performance Tests
1. CPU usage during recording (should be < 10%)
2. Memory usage (should be stable)
3. File size comparison
4. Recording latency

---

## Migration Path

### Backwards Compatibility
- Keep WAV as default for existing users
- Provide migration guide
- Support both old and new workflows

### Rollout Plan
1. **Week 1**: Implement core infrastructure (Phase 1)
2. **Week 2**: Update APIs and CLI (Phase 2-3)
3. **Week 3**: Testing and bug fixes
4. **Week 4**: Documentation and release

---

## Benefits Summary

### Before (Current)
- 30-minute recording: 660 MB temp files + 43 MB final = 703 MB peak disk usage
- Post-processing time: 10-30 seconds
- User must wait for processing

### After (New)
- 30-minute recording: 86 MB final files (2x M4A channels)
- Post-processing time: 0 seconds (or 2-3 seconds for optional merge)
- **87% reduction in disk usage**
- **Immediate results**
- **No waiting**

---

## Risk Assessment

### Low Risk
- ✅ FFmpeg is already a dependency
- ✅ Trait abstraction allows easy rollback
- ✅ WAV recording remains unchanged (fallback option)

### Medium Risk
- ⚠️ FFmpeg process management (mitigation: health monitoring)
- ⚠️ Buffer backpressure handling (mitigation: buffered writes)

### High Risk
- ❌ None identified

---

## Next Steps

1. **Approve this plan**
2. **Create feature branch**: `feature/streaming-compression`
3. **Implement Phase 1**: Core infrastructure
4. **Test on Windows**: Verify FFmpeg pipe behavior
5. **Iterate and refine**

---

## Open Questions

1. **Should we keep WAV as default or switch to M4A?**
   - Recommendation: Switch to M4A (better UX)

2. **What bitrates should we support?**
   - Recommendation: 128kbps, 192kbps (default), 256kbps, 320kbps

3. **Should we merge automatically or keep separate files?**
   - Recommendation: Keep separate initially, add merge option later

4. **Should we add Opus support in Phase 1?**
   - Recommendation: No, focus on M4A first (better compatibility)
