# Audio Recorder Manager - Architecture

## Overview

Audio Recorder Manager is a Rust-based monorepo workspace with a shared core library and two frontends:
- **CLI**: Command-line interface for standalone audio recording
- **Tauri UI**: Desktop GUI application with Svelte frontend

## Workspace Structure

```
audio-recorder-manager/
├── crates/
│   ├── core/              # Shared library (audio-recorder-manager-core)
│   │   ├── src/
│   │   │   ├── commands/  # Business logic for record, stop, recover, status
│   │   │   ├── status/    # Status file management
│   │   │   ├── transcription/ # Audio transcription features
│   │   │   ├── recorder.rs    # Audio capture and FFmpeg integration
│   │   │   ├── config.rs      # Configuration management
│   │   │   ├── devices.rs     # Audio device detection
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── cli/               # CLI binary (audio-recorder-manager-cli)
│   │   ├── src/
│   │   │   └── main.rs    # Thin wrapper - delegates to core
│   │   └── Cargo.toml
│   │
│   └── tauri-app/         # Tauri application (audio-recorder-manager-tauri)
│       ├── src/
│       │   └── main.rs    # Tauri commands - delegates to core
│       ├── ui/            # Svelte frontend
│       │   ├── src/
│       │   │   └── lib/components/
│       │   └── package.json
│       └── Cargo.toml
│
├── storage/               # Shared runtime data (gitignored)
│   ├── recordings/        # Output audio files (.wav, .m4a)
│   ├── signals/           # Stop signals for manual recordings
│   ├── status/            # Recording status JSON files
│   └── transcriptions/    # Transcription results
│
├── tests/                 # Integration tests
│   └── fixtures/          # Test data and configs
│
├── docs/                  # Documentation
│   ├── architecture.md    # This file
│   ├── tauri-setup.md     # UI development guide
│   └── reference-python-version/ # Original Python implementation
│
├── examples/              # Example scripts and configs
│
├── Cargo.toml             # Workspace root
└── README.md
```

## Core Library Design

### Separation of Concerns

The `core` crate is organized into focused modules:

#### Commands Module (`commands/`)
- **record.rs**: Orchestrates dual-channel recording, session management
- **stop.rs**: Handles stop signals for manual recording mode
- **recover.rs**: Recovers interrupted recordings, merges temp files
- **status.rs**: Enumerates audio devices

#### Platform-Specific Audio (`wasapi_*`)
- **wasapi_loopback.rs**: Windows system audio capture (WASAPI loopback)
- **wasapi_microphone.rs**: Windows microphone capture
- Both use Windows APIs directly for low-latency recording

#### Cross-Platform Components
- **recorder.rs**: High-level recording API, quality presets, FFmpeg integration
- **devices.rs**: Cross-platform device detection using `cpal`
- **config.rs**: Configuration with workspace-aware storage paths
- **domain.rs**: Core types (SessionId, AudioFormat, RecordingDuration)

### Configuration System

The `RecorderConfig` automatically locates the workspace root storage directory:

```rust
impl RecorderConfig {
    fn get_workspace_storage_dir() -> PathBuf {
        // Searches up the directory tree for Cargo.toml with [workspace]
        // Returns workspace_root/storage
        // Both CLI and Tauri use the same storage location
    }
}
```

**Benefits:**
- Single source of truth for all recordings
- CLI and UI can access the same files
- No data duplication

## Frontend Architecture

### CLI (crates/cli)

Minimal wrapper around the core library:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    cli::run(std::env::args().collect()).await
}
```

All logic is in `core::cli` module.

### Tauri UI (crates/tauri-app)

**Backend (`src/main.rs`):**
- Tauri commands that delegate to core library
- File watching for real-time status updates
- Transcription API integration

**Frontend (`ui/src`):**
- Svelte components
- Real-time recording status display
- Device management UI
- Recording list and playback
- Transcription viewer

## Data Flow

### Recording Lifecycle

```
User Request (CLI or UI)
    ↓
Command Handler (core::commands::record)
    ↓
Recorder (core::recorder)
    ↓
Platform Audio Capture (wasapi_loopback + wasapi_microphone)
    ↓
Temp WAV Files (storage/recordings/*_loopback.wav, *_mic.wav)
    ↓
FFmpeg Merge (dual-mono stereo)
    ↓
Final Output (storage/recordings/*.wav or *.m4a)
    ↓
Status Files (storage/status/*.json) - Updated every second
```

### Status Updates

Both frontends monitor `storage/status/{session_id}.json` for real-time progress:

```json
{
  "status": "recording",
  "session_id": "rec-20251116_123456",
  "elapsed": 15,
  "progress": 50,
  "loopback_frames": 720000,
  "loopback_has_audio": true,
  "mic_frames": 715000,
  "mic_has_audio": true
}
```

## Build System

### Cargo Workspace

The workspace configuration in root `Cargo.toml`:
- Shares dependencies across all crates
- Enables LTO and optimization for release builds
- Single `Cargo.lock` for reproducible builds

### Building

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p audio-recorder-manager-cli --release
cargo build -p audio-recorder-manager-tauri --release

# CLI binary location
# target/release/audio-recorder-manager.exe (or audio-recorder-manager on Unix)

# Tauri app
cd crates/tauri-app
npm install
npm run tauri build
```

## Key Design Decisions

### Why Monorepo?
- **Code Reuse**: Core library shared between CLI and UI
- **Consistency**: Single source of truth for business logic
- **Testing**: Shared test infrastructure
- **Versioning**: All components stay in sync

### Why Workspace Storage?
- **Data Sharing**: CLI and UI access the same recordings
- **Simplicity**: One storage location, no sync needed
- **Portability**: Easy to backup or relocate

### Why FFmpeg for Merging?
- **Robustness**: Handles sample rate mismatches automatically
- **Quality**: Professional-grade audio processing
- **Flexibility**: Easy to add formats (MP3, FLAC, etc.)

## Performance Characteristics

- **Zero-overhead audio capture**: Rust's type system ensures no runtime cost
- **Lock-free frame counting**: Atomic operations for concurrent access
- **Async I/O**: Non-blocking with Tokio runtime
- **Direct WAV writing**: No intermediate buffering
- **Efficient FFmpeg spawning**: Process reuse where possible

## Future Enhancements

Potential improvements:
1. **Cross-platform support**: macOS and Linux audio backends
2. **Plugin system**: Custom audio processors
3. **Cloud storage**: Upload recordings to S3, Dropbox, etc.
4. **Real-time visualization**: Waveform display during recording
5. **Batch transcription**: Queue multiple files
6. **Audio effects**: Noise reduction, normalization, etc.
