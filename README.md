# Audio Recorder Manager

A high-performance audio recording system built with Rust, featuring both a command-line interface and a desktop GUI application. Organized as a Cargo workspace monorepo with a shared core library.

## Project Structure

This is a **Cargo workspace** with three crates:
- **`crates/core`**: Shared library (`audio-recorder-manager-core`) - core recording logic
- **`crates/cli`**: Command-line interface (`audio-recorder-manager-cli`)
- **`crates/tauri-app`**: Desktop GUI application with Svelte frontend

Both CLI and GUI use the same storage location (`storage/`) for recordings and transcriptions.

## Acknowledgments

This project was inspired by and based on the Python implementation from [MeetingScribe](https://github.com/arthurhrk/meetingscribe) by Arthur Andrade. The original Python code served as the foundation for this Rust implementation, which maintains full CLI compatibility while providing significant performance improvements.

## Features

### Core Recording Features
- **Dual-channel recording** on Windows (system audio + microphone simultaneously)
- Intelligent audio merging with FFmpeg (dual-mono stereo: L=system, R=microphone)
- **Recovery mode** for interrupted recordings - automatically completes merge and conversion
- Automatic fallback when microphone is unavailable
- Real-time status updates during recording showing both channels
- JSON-based status files for frontend integration
- Manual recording mode with dedicated stop command
- Multiple quality presets (quick, standard, professional, high)
- Professional quality audio (48kHz, 16-bit, stereo)
- M4A format support with automatic WAV conversion
- Compatible with existing Python CLI interface

### Desktop GUI Features (Tauri App)
- **Embedded Audio Player** with visual waveform display
  - Real-time waveform visualization using FFmpeg analysis
  - Play/pause controls with timeline scrubbing
  - Duration display and progress tracking
  - Integrated directly into recording detail view

- **Modern Recording Interface**
  - Clean, streamlined recording panel with preset duration buttons
  - Live audio level monitoring for both loopback and microphone
  - Real-time visual feedback during recording setup
  - Quality and format selection with detailed specifications

- **Enhanced Recording Management**
  - Modern card-based list view with search and filtering
  - Inline rename functionality - double-click to edit
  - Quick actions: play, delete, transcribe
  - Visual status indicators for transcriptions

- **System Tray Integration**
  - Minimize to tray instead of closing the app
  - Quick Record submenu (30s, 1m, 5m, 10m presets)
  - Show/Hide window toggle
  - Direct access to recordings folder
  - Left-click tray icon to toggle window visibility

## Quick Start

### Download Pre-built Binary

**Windows users** can download the pre-built executable from the [latest release](https://github.com/dougfernando/audio-recorder-manager/releases/latest):

1. Download `audio-recorder-manager.exe`
2. Run it directly - no installation required!

```bash
# Start recording
audio-recorder-manager.exe record 30 wav
```

### Build from Source

## Requirements

### All Platforms
- **FFmpeg**: Required for dual-channel merging and M4A conversion
  - Windows: Download from https://ffmpeg.org/download.html or https://www.gyan.dev/ffmpeg/builds/
  - Linux: `sudo apt-get install ffmpeg`
  - macOS: `brew install ffmpeg`

### Windows (Build Requirements)
- Visual Studio Build Tools with "Desktop development with C++" workload
  - Download from: https://visualstudio.microsoft.com/downloads/
  - Or use `rustup target add x86_64-pc-windows-gnu` for MinGW toolchain

### Linux (Build Requirements)
```bash
sudo apt-get install libasound2-dev pkg-config
```

### macOS (Build Requirements)
No additional build dependencies required.

## Installation

### Build from Source

```bash
# Build entire workspace (CLI + Tauri)
cargo build --release

# Build only CLI
cargo build -p audio-recorder-manager-cli --release

# Build only Tauri app
cargo build -p audio-recorder-manager-tauri --release
```

The compiled CLI binary will be in `target/release/audio-recorder-manager.exe` (Windows) or `target/release/audio-recorder-manager` (Linux/macOS).

For Tauri GUI, see [docs/tauri-setup.md](docs/tauri-setup.md).

## Usage

### Desktop GUI (Tauri App)

The Tauri application provides a modern desktop interface for audio recording:

```bash
# Development mode (with hot reload)
cd crates/tauri-app
cargo tauri dev

# Build production app
cargo tauri build
```

**Key Features:**
- Visual recording controls with live audio level monitoring
- Browse and play recordings with embedded audio player and waveform
- Inline rename - double-click any recording name to edit
- System tray integration - app minimizes to tray instead of closing
- Quick recording from tray menu (30s, 1m, 5m, 10m presets)
- Transcription support with visual status indicators

The GUI uses the same storage location as the CLI, so recordings are accessible from both interfaces.

### Command Line Interface (CLI)

This Rust version maintains full compatibility with the Python CLI interface:

```bash
# Start recording for 30 seconds (default format: wav, quality: professional)
audio-recorder-manager record 30

# Record with specific format and quality
audio-recorder-manager record 30 m4a standard

# Manual mode - record until stop command
audio-recorder-manager record -1 wav quick

# Stop the latest active recording
audio-recorder-manager stop

# Stop a specific recording session
audio-recorder-manager stop rec-20250109_120000

# Recover all incomplete recordings (from interrupted sessions)
audio-recorder-manager recover

# Recover a specific session
audio-recorder-manager recover rec-20250109_120000

# Recover and convert to M4A format
audio-recorder-manager recover rec-20250109_120000 m4a

# Check system audio devices
audio-recorder-manager status
```

### Dual-Channel Recording (Windows Only)

The recorder automatically captures **both system audio and microphone** simultaneously:

- **Output format**: Stereo WAV/M4A with dual-mono layout
  - **Left channel**: System audio (speakers/loopback) - captures meeting participants
  - **Right channel**: Microphone input - captures your voice
- **Smart merging**: Uses FFmpeg to intelligently merge based on audio detection
  - Both active: Dual-mono stereo (L=system, R=mic)
  - System only: Stereo duplication (listening-only meeting)
  - Mic only: Stereo duplication (rare case)
  - Neither: Valid silent stereo file
- **Automatic fallback**: If microphone is unavailable, continues with system audio only
- **Sample rate alignment**: FFmpeg automatically handles mismatches (e.g., 44.1kHz mic + 48kHz system)

**Perfect for recording Teams, Google Meet, Zoom, and other online meetings!**

Terminal output example:
```
[Recording] Progress: 50% | Elapsed: 15s / 30s | Loopback: 1440 frames (Audio) | Mic: 1425 frames (Audio)
[Recording] Completed successfully!
[Merging] Merging audio channels...
[Merging] Successfully merged audio channels!
```

### Recovery Mode

If a recording is interrupted (e.g., program killed, system crash), the temporary WAV files (`*_loopback.wav` and `*_mic.wav`) are preserved. Use the `recover` command to complete the merge and conversion:

```bash
# Recover all incomplete recordings
audio-recorder-manager recover

# Recover specific session and convert to M4A
audio-recorder-manager recover rec-20250109_120000 m4a
```

Recovery output example:
```json
{
  "status": "success",
  "data": {
    "message": "Successfully recovered 1 recording(s)",
    "recovered": [
      {
        "session_id": "rec-20250109_120000",
        "output_file": "recording_20250109_120000.wav",
        "output_path": "storage/recordings/recording_20250109_120000.wav"
      }
    ]
  }
}
```

The recover command will:
1. Scan for orphaned temporary files (`*_loopback.wav`, `*_mic.wav`)
2. Merge them using FFmpeg (same as normal recording)
3. Optionally convert to M4A format
4. Clean up temporary files after successful recovery

### Quality Presets

- **quick**: 16kHz, Stereo - Fast encoding, smaller files
- **standard**: 44.1kHz, Stereo - CD quality
- **professional**: 48kHz, Stereo - Studio quality (default)
- **high**: 96kHz, Stereo - Hi-res audio

### Output Format

All commands return JSON for easy integration with frontends:

```json
{
  "status": "success",
  "data": {
    "session_id": "rec-20250107_123456",
    "file_path": "storage/recordings/recording_20250107_123456.wav",
    "filename": "recording_20250107_123456.wav",
    "duration": 30,
    "message": "Recording started successfully"
  }
}
```

### Status Files

Recording status is written to `storage/status/{session_id}.json` every second:

```json
{
  "status": "recording",
  "session_id": "rec-20250107_123456",
  "filename": "recording_20250107_123456.wav",
  "duration": 30,
  "elapsed": 15,
  "progress": 50,
  "quality": "professional",
  "device": "Default Audio Device",
  "sample_rate": 48000,
  "channels": 2,
  "frames_captured": 1500,
  "has_audio": true
}
```

## Architecture

The project is organized as a **Cargo workspace monorepo**:

```
audio-recorder-manager/
├── crates/
│   ├── core/              # Shared library (all business logic)
│   │   ├── commands/      # record, stop, recover, status
│   │   ├── transcription/ # Audio transcription
│   │   ├── status/        # Status file management
│   │   └── ...
│   ├── cli/               # CLI binary (thin wrapper)
│   └── tauri-app/         # Tauri GUI + Svelte frontend
├── storage/               # Shared storage (gitignored)
│   ├── recordings/        # Output files
│   ├── status/            # Recording status
│   ├── signals/           # Stop signals
│   └── transcriptions/    # Transcripts
├── docs/                  # Documentation
│   ├── architecture.md    # Detailed design docs
│   ├── migration-guide.md # Reorganization guide
│   └── tauri-setup.md     # UI development
└── examples/              # Example scripts

```

### Design Principles

- **Monorepo**: Single source of truth with shared core library
- **Separation of Concerns**: Each module has a single, clear responsibility
- **Unified Storage**: CLI and GUI share the same recordings directory
- **Type Safety**: Strong typing prevents runtime errors
- **Async/Await**: Non-blocking I/O with Tokio runtime

See [docs/architecture.md](docs/architecture.md) for detailed documentation.

## Performance Improvements over Python

- Zero-overhead audio capture with Rust's type system
- Lock-free atomic operations for frame counting
- Efficient memory management without GC pauses
- Native threading with Tokio async runtime
- Direct WAV file writing without intermediate buffers

## Development

```bash
# Check entire workspace
cargo check

# Run CLI with logging
RUST_LOG=debug cargo run -p audio-recorder-manager-cli -- record 5 wav

# Run tests
cargo test -p audio-recorder-manager-core

# Build optimized release
cargo build --release

# Tauri development
cd crates/tauri-app/ui && npm install
cd .. && cargo tauri dev
```

See [docs/migration-guide.md](docs/migration-guide.md) for detailed development workflow.

## License

MIT
