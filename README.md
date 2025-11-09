# Audio Recorder Manager

A high-performance command-line audio recorder manager built with Rust, converted from the Python version for improved performance and reliability.

## Acknowledgments

This project was inspired by and based on the Python implementation from [MeetingScribe](https://github.com/arthurhrk/meetingscribe) by Arthur Henrique Della Fraga. The original Python code served as the foundation for this Rust implementation, which maintains full CLI compatibility while providing significant performance improvements.

## Features

- Record audio from available devices (system audio/loopback on Windows)
- Real-time status updates during recording
- JSON-based status files for frontend integration
- Manual recording mode with stop signals
- Professional quality audio (48kHz, 16-bit, stereo)
- Compatible with existing Python CLI interface

## Requirements

### Windows
- Visual Studio Build Tools with "Desktop development with C++" workload
  - Download from: https://visualstudio.microsoft.com/downloads/
  - Or use `rustup target add x86_64-pc-windows-gnu` for MinGW toolchain

### Linux
```bash
sudo apt-get install libasound2-dev pkg-config
```

### macOS
No additional dependencies required.

## Installation

```bash
cargo build --release
```

The compiled binary will be in `target/release/audio-recorder-manager.exe` (Windows) or `target/release/audio-recorder-manager` (Linux/macOS).

## Usage

This Rust version maintains full compatibility with the Python CLI interface:

```bash
# Start recording for 30 seconds (default)
audio-recorder-manager record 30 wav

# Manual mode - record until stop signal
audio-recorder-manager record -1 wav

# Check system audio devices
audio-recorder-manager status
```

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

### Manual Stop

To stop a manual recording, create a stop signal file:
```bash
# File: storage/signals/{session_id}.stop
touch storage/signals/rec-20250107_123456.stop
```

## Architecture

- **devices.rs**: Audio device detection and management using `cpal`
- **recorder.rs**: Audio recording with real-time status updates
- **main.rs**: CLI interface matching Python implementation

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
