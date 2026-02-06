# CLAUDE.md - AI Assistant Guide for Audio Recorder Manager

## Project Overview

Audio Recorder Manager is a high-performance Rust audio recording system with dual interfaces: a CLI and a Tauri desktop GUI (Svelte frontend). It captures system audio + microphone simultaneously on Windows via WASAPI, merges channels with FFmpeg, and supports transcription via Gemini API. Version 0.7.0.

## Codebase Structure

```
audio-recorder-manager/          # Cargo workspace root
├── crates/
│   ├── core/                    # Shared library (audio-recorder-manager-core)
│   │   ├── src/
│   │   │   ├── commands/        # record.rs, stop.rs, recover.rs, status.rs, cancel.rs
│   │   │   ├── status/          # JSON-based status file management
│   │   │   ├── transcription/   # Gemini API transcription integration
│   │   │   ├── recorder.rs      # Core audio recording logic (~1,549 lines)
│   │   │   ├── config.rs        # Configuration & workspace-aware storage paths
│   │   │   ├── domain.rs        # Core types: SessionId, AudioFormat, RecordingDuration
│   │   │   ├── devices.rs       # Cross-platform audio device detection (cpal)
│   │   │   ├── logging.rs       # Dual-output logging (file + terminal)
│   │   │   ├── wasapi_loopback.rs   # Windows system audio capture
│   │   │   ├── wasapi_microphone.rs # Windows microphone capture
│   │   │   └── audio_monitor.rs     # Real-time audio level monitoring
│   │   └── tests/               # Integration tests
│   ├── cli/                     # CLI binary (audio-recorder-manager-cli, ~230 lines)
│   │   └── src/
│   │       ├── main.rs          # Entry point - thin wrapper
│   │       └── cli.rs           # Clap command parsing
│   └── tauri-app/               # Tauri GUI (audio-recorder-manager-tauri)
│       ├── src/
│       │   ├── main.rs          # Tauri command handlers (~1,718 lines)
│       │   ├── splash_screen.rs # Splash screen UI
│       │   ├── state.rs         # AppState definition
│       │   └── dto.rs           # Data transfer objects
│       ├── ui/                  # Svelte frontend
│       │   ├── src/
│       │   │   ├── App.svelte
│       │   │   └── lib/components/  # 13 Svelte components
│       │   ├── package.json
│       │   └── vite.config.js
│       └── tauri.conf.json
├── storage/                     # Runtime data (gitignored)
│   ├── recordings/              # Output audio files
│   ├── status/                  # Recording status JSON files
│   ├── signals/                 # Stop signals for manual recording
│   └── transcriptions/          # Transcription results
├── docs/
│   ├── architecture.md          # Detailed design documentation
│   ├── tauri-setup.md           # UI development guide
│   ├── migration-guide.md       # Crate reorganization guide
│   └── reference-python-version/ # Original Python implementation
├── Cargo.toml                   # Workspace definition
└── claude.md                    # Antivirus safety guidelines (MUST READ)
```

## Build Commands

```bash
# Type checking (fastest feedback loop)
cargo check

# Build entire workspace
cargo build --release

# Build individual crates
cargo build -p audio-recorder-manager-cli --release
cargo build -p audio-recorder-manager-tauri --release

# Run CLI with debug logging
RUST_LOG=debug cargo run -p audio-recorder-manager-cli -- record 5 wav

# Run tests
cargo test -p audio-recorder-manager-core

# Run ignored/manual tests (e.g., transcription tests requiring API keys)
cargo test -p audio-recorder-manager-core --test transcription_test -- --ignored

# Tauri GUI development (hot reload)
cd crates/tauri-app/ui && npm install && cd .. && cargo tauri dev

# Tauri production build
cd crates/tauri-app && cargo tauri build
```

**Build profiles:**
- `dev`: Optimized dependencies (opt-level=3), incremental compilation
- `release`: LTO enabled, single codegen unit, stripped binaries
- `release-dev`: Faster builds with LTO disabled

## Testing

- Test framework: `#[tokio::test]` (async tests with Tokio runtime)
- Tests live in `crates/core/tests/` and inline in source modules
- Some tests are `#[ignore]` because they require external services (Gemini API, real audio devices)
- Test helper: `init_test_logging()` for enabling log output in tests

## Code Conventions

### Architecture
- **Monorepo with shared core**: All business logic lives in `crates/core`. CLI and Tauri are thin wrappers.
- **Command pattern**: Each operation (record, stop, recover, status) is a module in `commands/` with an `execute()` async function.
- **Unified storage**: CLI and GUI share the same `storage/` directory, discovered by walking up to find `Cargo.toml` with `[workspace]`.

### Rust Patterns
- **Error handling**: `thiserror` for custom error types (`RecorderError`), `anyhow` for ad-hoc errors. Custom `Result<T>` alias.
- **Domain types**: Strong typing with newtypes — `SessionId`, `AudioFormat` (enum), `RecordingDuration` (enum), `RecordingQuality` (struct).
- **Async**: Tokio runtime throughout. Use `tokio::fs` for file I/O, `tokio::process::Command` for subprocess spawning.
- **Logging**: `tracing` crate with dual-output (file + terminal). Logs go to `%APPDATA%\audio-recorder-manager\logs\`.
- **JSON output**: All CLI commands return structured JSON (`{"status": "success", "data": {...}}`).
- **Platform code**: Windows-specific WASAPI code is in `wasapi_*.rs` files. Unsafe blocks must be minimal and documented.

### Frontend (Tauri/Svelte)
- Svelte 4 with Vite 5
- Tailwind CSS for styling
- `lucide-svelte` for icons
- Tauri commands defined with `#[tauri::command]` in `main.rs`
- Frontend communicates with backend via Tauri's invoke API

### Dependencies
Key workspace dependencies (defined in root `Cargo.toml`):
- `tokio` 1.41 — async runtime
- `serde`/`serde_json` — serialization
- `cpal` 0.15 — cross-platform audio device abstraction
- `hound` 3.5 — WAV file I/O
- `clap` 4.5 — CLI argument parsing
- `reqwest` 0.12 — HTTP client (Gemini API)
- `windows` 0.58 — Windows API bindings (WASAPI, COM)
- `tauri` 2.1 — desktop app framework

## Critical: Antivirus Safety Guidelines

**This is a legitimate audio recording app. Code that resembles malware behavior will trigger antivirus false positives and cause user harm.**

See `claude.md` for the full policy. Key rules:

### Prohibited
- Process enumeration/monitoring (`tasklist`, `ps`, `Get-Process`)
- Hidden process execution without documented justification
- Keyboard/mouse logging, screenshots, system-wide hooks
- Undisclosed network connections
- Writing to system directories or dropping executables

### Approved
- WASAPI audio capture (COM, `IMMDeviceEnumerator`, `IAudioClient`)
- FFmpeg spawning with `CREATE_NO_WINDOW` (prevents console flash)
- File I/O within `storage/` directories only
- Explicit user-requested network calls (transcription API)

### Before Any Code Change, Verify:
- No external process querying
- No hidden windows (unless strictly necessary and documented)
- No system-wide hooks or injection
- All `unsafe` blocks are minimal and documented with justification
- All Windows API calls have clear purpose documented

## Development Workflow

1. `cargo check` — fast type checking before committing
2. `cargo test -p audio-recorder-manager-core` — run core tests
3. `cargo build --release` — full optimized build
4. For Tauri UI work: `cargo tauri dev` from `crates/tauri-app/`

## Key Files for Common Tasks

| Task | Files |
|------|-------|
| Add a new CLI command | `crates/core/src/commands/`, `crates/cli/src/cli.rs` |
| Modify recording logic | `crates/core/src/recorder.rs` |
| Change audio capture | `crates/core/src/wasapi_loopback.rs`, `wasapi_microphone.rs` |
| Update configuration | `crates/core/src/config.rs` |
| Add domain types | `crates/core/src/domain.rs` |
| Modify Tauri commands | `crates/tauri-app/src/main.rs` |
| Update UI components | `crates/tauri-app/ui/src/lib/components/` |
| Update transcription | `crates/core/src/transcription/` |
| Status file format | `crates/core/src/status/` |
