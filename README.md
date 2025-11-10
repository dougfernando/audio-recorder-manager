# Audio Recorder Manager

A high-performance command-line audio recorder manager built with Rust, converted from the Python version for improved performance and reliability.

## Acknowledgments

This project was inspired by and based on the Python implementation from [MeetingScribe](https://github.com/arthurhrk/meetingscribe) by Arthur Andrade. The original Python code served as the foundation for this Rust implementation, which maintains full CLI compatibility while providing significant performance improvements.

## Features

- **Dual-channel recording** on Windows (system audio + microphone simultaneously)
- Intelligent audio merging with FFmpeg (dual-mono stereo: L=system, R=microphone)
- Automatic fallback when microphone is unavailable
- Real-time status updates during recording showing both channels
- JSON-based status files for frontend integration
- Manual recording mode with dedicated stop command
- Multiple quality presets (quick, standard, professional, high)
- Professional quality audio (48kHz, 16-bit, stereo)
- M4A format support with automatic WAV conversion
- Compatible with existing Python CLI interface

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

```bash
cargo build --release
```

The compiled binary will be in `target/release/audio-recorder-manager.exe` (Windows) or `target/release/audio-recorder-manager` (Linux/macOS).

## Usage

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

The codebase follows a modular architecture for maintainability and scalability:

### Core Modules

- **main.rs**: Application entry point (18 lines)
- **cli.rs**: Command-line argument parsing and routing
- **commands/**: Command implementations
  - **record.rs**: Dual-channel recording orchestration and session management
  - **stop.rs**: Stop signal creation and active session detection
  - **status.rs**: Audio device enumeration
- **config.rs**: Configuration management
- **devices.rs**: Audio device detection using `cpal`
- **domain.rs**: Core domain types (SessionId, AudioFormat, RecordingDuration)
- **recorder.rs**: Audio capture, quality presets, and FFmpeg integration
- **status/**: Status file management
- **wasapi_loopback/**: Windows WASAPI loopback recording (system audio)
- **wasapi_microphone/**: Windows WASAPI microphone recording (input audio)

### Design Principles

- **Separation of Concerns**: Each module has a single, clear responsibility
- **Command Pattern**: Each command is self-contained in its own module
- **Type Safety**: Strong typing prevents runtime errors
- **Async/Await**: Non-blocking I/O with Tokio runtime

## Performance Improvements over Python

- Zero-overhead audio capture with Rust's type system
- Lock-free atomic operations for frame counting
- Efficient memory management without GC pauses
- Native threading with Tokio async runtime
- Direct WAV file writing without intermediate buffers

## Development

```bash
# Check for compilation errors
cargo check

# Run with logging
RUST_LOG=debug cargo run -- record 5 wav

# Run tests
cargo test

# Build optimized release
cargo build --release
```

## License

MIT
