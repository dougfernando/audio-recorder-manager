# Research: Direct Compressed Audio Recording on Windows

## Current Implementation Analysis

### Current Workflow
1. **Capture**: WASAPI → Raw PCM audio buffers
2. **Write**: `hound` crate → Temporary WAV files (16-bit PCM)
3. **Post-process**: FFmpeg → Merge channels + M4A encoding (AAC 192kbps)

### Current Issues
- **Storage**: WAV files are ~11 MB/min (professional quality)
- **Processing Time**: Post-recording merge + encoding step required
- **Delay**: User must wait for processing to complete before accessing final file

### Performance Numbers
- WAV writing: Real-time (no overhead)
- M4A encoding: 20-50x real-time (fast but adds delay)
- Combined merge+encode: 50-70% faster than separate operations

---

## Proposed Solutions for Direct Compressed Recording

### ✅ **OPTION 1: Real-time FFmpeg Pipe (RECOMMENDED)**

#### Description
Stream PCM audio directly from WASAPI to FFmpeg stdin, encode to M4A in real-time during recording.

#### Architecture
```
WASAPI → PCM Buffer → FFmpeg stdin → M4A file
         (memory)      (pipe)        (disk)
```

#### Implementation Approach
```rust
// Simplified pseudo-code
fn start_recording() {
    // Start FFmpeg process with stdin pipe
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-f", "s16le",           // 16-bit PCM input
            "-ar", "48000",          // Sample rate
            "-ac", "2",              // Stereo
            "-i", "pipe:0",          // Read from stdin
            "-c:a", "aac",           // AAC codec
            "-b:a", "192k",          // Bitrate
            "output.m4a"
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = ffmpeg.stdin.take().unwrap();

    // In WASAPI capture loop
    loop {
        let pcm_data = capture_audio_from_wasapi();
        stdin.write_all(&pcm_data)?; // Stream to FFmpeg
    }
}
```

#### Pros
- ✅ No temporary WAV files (saves disk I/O)
- ✅ No post-processing delay (encoding happens during recording)
- ✅ Immediate M4A output when recording stops
- ✅ Works with existing FFmpeg dependency
- ✅ Supports multiple formats (M4A, MP3, OGG, OPUS)
- ✅ Can still do dual-channel recording (two FFmpeg processes)

#### Cons
- ⚠️ Requires careful buffer management (backpressure handling)
- ⚠️ No recovery option if FFmpeg crashes mid-recording
- ⚠️ Slightly more complex error handling
- ⚠️ CPU encoding overhead during recording (but modern CPUs handle this easily)

#### Complexity: **Medium**
#### Performance Impact: **Minimal** (AAC encoding is ~5-10% CPU on modern processors)
#### Compatibility: **Excellent** (FFmpeg is already used)

---

### ✅ **OPTION 2: Media Foundation AAC Encoder (Windows Native)**

#### Description
Use Windows Media Foundation API for native hardware-accelerated AAC encoding.

#### Architecture
```
WASAPI → PCM Buffer → Media Foundation Transform → M4A file
         (memory)      (AAC encoder)              (disk)
```

#### Implementation Approach
```rust
// Windows-specific using windows-rs crate
use windows::Win32::Media::MediaFoundation::*;

fn start_recording() {
    // Initialize Media Foundation
    MFStartup(MF_VERSION, MFSTARTUP_FULL)?;

    // Create AAC encoder transform
    let encoder: IMFTransform = CoCreateInstance(
        &CLSID_AACMFTEncoder,
        None,
        CLSCTX_INPROC_SERVER
    )?;

    // Configure input/output types
    // Process PCM samples through encoder
    // Write compressed samples to file
}
```

#### Pros
- ✅ Native Windows API (no external dependencies)
- ✅ Hardware acceleration support (Intel Quick Sync, NVIDIA NVENC)
- ✅ Lower CPU usage than software encoding
- ✅ Supports M4A, WMA, MP3 (various codecs)
- ✅ Real-time encoding guaranteed by OS
- ✅ Part of Windows since Vista (excellent compatibility)

#### Cons
- ⚠️ Windows-only (but that's the focus)
- ⚠️ More complex COM/Media Foundation code
- ⚠️ Requires `windows` crate with Media Foundation features
- ⚠️ Less format flexibility than FFmpeg
- ⚠️ M4A container writing requires additional code (MP4 muxer)

#### Complexity: **High**
#### Performance Impact: **Very Low** (hardware accelerated when available)
#### Compatibility: **Windows 7+**

---

### ✅ **OPTION 3: Opus Real-time Encoding (Best Quality/Size Ratio)**

#### Description
Use `opus` crate for real-time Opus encoding (best compression for voice/music).

#### Architecture
```
WASAPI → PCM Buffer → libopus encoder → OGG/Opus file
         (memory)      (Rust crate)      (disk)
```

#### Implementation Approach
```rust
use opus::{Encoder, Application};
use ogg::PacketWriter;

fn start_recording() {
    // Create Opus encoder
    let mut encoder = Encoder::new(
        48000,                    // Sample rate
        opus::Channels::Stereo,   // Channels
        Application::Audio        // Application type
    )?;

    encoder.set_bitrate(opus::Bitrate::Bits(192000))?;

    // Create OGG container writer
    let mut ogg_writer = PacketWriter::new(file);

    // In capture loop
    loop {
        let pcm = capture_audio();
        let compressed = encoder.encode_vec(&pcm, 4000)?;
        ogg_writer.write_packet(compressed)?;
    }
}
```

#### Pros
- ✅ Best compression ratio (better than AAC/MP3)
- ✅ Lower latency than AAC
- ✅ Excellent audio quality at lower bitrates
- ✅ Pure Rust implementation available
- ✅ Very low CPU usage
- ✅ Cross-platform (works on all OSes)
- ✅ Open source, royalty-free

#### Cons
- ⚠️ Not M4A format (outputs .opus or .ogg)
- ⚠️ Less universal compatibility than M4A (though widely supported)
- ⚠️ Requires new dependencies (`opus`, `ogg` crates)

#### Complexity: **Low-Medium**
#### Performance Impact: **Very Low** (Opus is highly optimized)
#### Compatibility: **Good** (Chrome, Firefox, VLC, modern players)

---

### ✅ **OPTION 4: MP3 LAME Encoding (Maximum Compatibility)**

#### Description
Use LAME MP3 encoder for maximum compatibility across all platforms.

#### Architecture
```
WASAPI → PCM Buffer → LAME encoder → MP3 file
         (memory)      (C library)    (disk)
```

#### Implementation Approach
```rust
// Using lame-sys or mp3lame-encoder crate
use mp3lame_encoder::{Builder, FlushNoGap};

fn start_recording() {
    let mut encoder = Builder::new().expect("Create encoder");
    encoder.set_num_channels(2)?;
    encoder.set_sample_rate(48000)?;
    encoder.set_brate(192)?; // 192 kbps
    encoder.set_quality(2)?; // High quality

    let mut mp3_file = File::create("output.mp3")?;

    loop {
        let pcm = capture_audio();
        let mp3_frames = encoder.encode(&pcm)?;
        mp3_file.write_all(&mp3_frames)?;
    }
}
```

#### Pros
- ✅ Universal compatibility (all devices support MP3)
- ✅ Well-established format
- ✅ Low CPU usage
- ✅ No container format needed (simple file structure)
- ✅ Rust bindings available

#### Cons
- ⚠️ Lower quality than AAC/Opus at same bitrate
- ⚠️ Patent issues (expired in most countries but check local laws)
- ⚠️ Larger files than AAC/Opus for same quality
- ⚠️ Not as modern as AAC/Opus

#### Complexity: **Low**
#### Performance Impact: **Low**
#### Compatibility: **Excellent** (universal)

---

### ❌ **OPTION 5: Windows Media Audio (WMA)** - Not Recommended

#### Description
Use Windows Media Encoder for WMA format.

#### Pros
- Native Windows format
- Hardware acceleration possible

#### Cons
- ❌ Poor cross-platform compatibility
- ❌ Inferior quality to AAC/Opus
- ❌ Less widely supported
- ❌ Legacy format

**Verdict**: Not recommended due to limited adoption and better alternatives.

---

## Dual-Channel Recording Considerations

### Challenge
Current system records two separate streams (loopback + microphone) with intelligent merging.

### Solutions for Compressed Recording

#### **Approach A: Dual Encoding + Post-Merge (Hybrid)**
```
Loopback → FFmpeg pipe → loopback.m4a  ┐
                                        ├─→ Quick merge → final.m4a
Mic      → FFmpeg pipe → mic.m4a       ┘
```
- Still requires brief post-processing
- Smaller intermediate files (compressed)
- Fast merge (remux operation, no re-encoding)

#### **Approach B: Pre-Mix PCM Streams**
```
Loopback PCM ┐
             ├─→ Mix in memory → FFmpeg pipe → final.m4a
Mic PCM      ┘
```
- Single compressed output
- No post-processing needed
- More complex real-time mixing logic
- Loses flexibility of per-channel analysis

#### **Approach C: Parallel Encoding + Metadata**
```
Both channels encoded separately, keep both files
Let user choose merge options later
```
- Maximum flexibility
- Two smaller files vs one WAV
- Post-processing when user wants merged output

---

## Recommended Implementation Strategy

### **Phase 1: Add Real-time FFmpeg Pipe Support** (Quick Win)

**Target**: 1-2 days development
**Impact**: Immediate elimination of post-processing delay for M4A

#### Changes Required:
1. Add `output_format` parameter: `wav`, `m4a`, `opus`, `mp3`
2. Create `StreamingEncoder` trait for different formats
3. Implement `FFmpegPipeEncoder` for M4A
4. Modify WASAPI recorders to write to encoder instead of WAV
5. Update status reporting for real-time encoding progress

#### Code Structure:
```rust
trait StreamingEncoder {
    fn write_samples(&mut self, pcm_data: &[i16]) -> Result<()>;
    fn finish(self) -> Result<()>;
}

struct FFmpegPipeEncoder {
    ffmpeg_process: Child,
    stdin: ChildStdin,
}

struct WavEncoder {
    wav_writer: WavWriter<BufWriter<File>>,
}
```

#### Benefits:
- ✅ No post-processing delay for compressed formats
- ✅ Maintains WAV option for lossless recording
- ✅ Uses existing FFmpeg dependency
- ✅ Relatively simple implementation

---

### **Phase 2: Add Native Opus Support** (Quality Boost)

**Target**: 1-2 days development
**Impact**: Better compression, lower CPU, smaller files

#### Changes Required:
1. Add `opus` and `ogg` crates to dependencies
2. Implement `OpusEncoder` struct
3. Add Opus quality presets
4. Update UI to support .opus format

#### Benefits:
- ✅ Better quality/size ratio than M4A
- ✅ Lower latency
- ✅ Cross-platform
- ✅ No FFmpeg needed for Opus

---

### **Phase 3: Optimize Dual-Channel Workflow** (Advanced)

**Target**: 2-3 days development
**Impact**: Improved efficiency for dual-channel scenarios

#### Options:
1. Add "merge strategy" setting (separate files, pre-mixed, post-merge)
2. Implement real-time PCM mixing
3. Fast remux operation for compressed files

---

## Performance Comparison

| Format | File Size (30min) | Encoding CPU | Post-Process | Compatibility |
|--------|------------------|--------------|--------------|---------------|
| **WAV (current)** | 330 MB | 0% | ~10-30s | Universal |
| **M4A (FFmpeg pipe)** | 43 MB | 5-10% | 0s | Excellent |
| **Opus** | 35 MB | 3-5% | 0s | Good |
| **MP3** | 50 MB | 4-8% | 0s | Universal |

---

## Recommendation Summary

### 🏆 **Best Overall: FFmpeg Pipe + M4A** (Option 1)
- Minimal code changes
- Uses existing infrastructure
- Immediate results
- Excellent compatibility

### 🥈 **Best Quality/Size: Opus** (Option 3)
- Best compression
- Lowest CPU
- Great for archival
- Good modern compatibility

### Implementation Priority:
1. **Phase 1**: FFmpeg pipe for M4A (quick win)
2. **Phase 2**: Add Opus support (quality improvement)
3. **Phase 3**: Keep WAV as option for lossless needs

### Hybrid Approach:
Allow users to choose format at recording time:
```
record 300 m4a     # Real-time compressed
record 300 opus    # Real-time Opus
record 300 wav     # Lossless (current behavior)
```

This gives maximum flexibility while eliminating post-processing delays for compressed formats.
