# GUI Development Guide

## Overview

This project supports two binaries:
1. **audio-recorder-manager** (CLI) - Command-line interface
2. **audio-recorder-gui** (GUI) - Graphical user interface (under development)

Both binaries share the same core business logic through the library crate.

## Project Structure

```
audio-recorder-manager/
├── src/
│   ├── lib.rs                    # Library exposing core functionality
│   ├── main.rs                   # CLI entry point
│   ├── cli.rs                    # CLI implementation
│   ├── gui/
│   │   ├── main.rs               # GUI entry point
│   │   ├── app.rs                # Main app component
│   │   ├── components/           # UI components
│   │   ├── state/                # Application state
│   │   └── services/             # Service layer
│   ├── commands/                 # Shared command implementations
│   ├── recorder.rs               # Shared recording logic
│   ├── config.rs                 # Shared configuration
│   └── domain.rs                 # Shared domain types
├── Cargo.toml                    # Defines both binaries
└── docs/
    ├── GUI_PLAN.md               # Detailed GUI implementation plan
    └── GUI_DEVELOPMENT.md        # This file
```

## Building

### Build CLI only (default)
```bash
cargo build --bin audio-recorder-manager
# Or simply
cargo build
```

### Build GUI (requires GPUI)
```bash
cargo build --bin audio-recorder-gui --features gui
```

### Build both
```bash
cargo build --all
```

### Release builds
```bash
# CLI release
cargo build --release --bin audio-recorder-manager

# GUI release
cargo build --release --bin audio-recorder-gui --features gui
```

## Running

### CLI
```bash
cargo run --bin audio-recorder-manager -- record 30 wav
```

### GUI
```bash
cargo run --bin audio-recorder-gui --features gui
```

## Development Status

### Implemented ✅
- [x] Dual binary structure
- [x] Shared library with core functionality
- [x] GUI module scaffolding
- [x] Basic GPUI application structure
- [x] Application state foundation

### In Progress 🚧
- [ ] UI Components (see GUI_PLAN.md)
- [ ] Service layer implementation
- [ ] File watching for status updates
- [ ] Theme system

### Planned 📋
- [ ] Complete Record Panel
- [ ] Monitor Panel with real-time updates
- [ ] History Panel with recording list
- [ ] Recovery Panel
- [ ] Settings Panel
- [ ] System tray integration
- [ ] Notifications

## Dependencies

### Shared Dependencies (CLI + GUI)
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `anyhow` - Error handling
- `chrono` - Date/time handling

### CLI-only Dependencies
- `clap` - Command-line argument parsing

### GUI-only Dependencies (feature-gated)
- `gpui` - UI framework
- Additional GUI dependencies will be added as needed

## Adding New Features

### To add a new core feature (used by both CLI and GUI):
1. Implement in `src/` (e.g., `commands/`, `recorder.rs`, etc.)
2. Export from `src/lib.rs` if needed by GUI
3. CLI uses it via the library
4. GUI uses it via the library

### To add a GUI-only feature:
1. Implement in `src/gui/`
2. Add to appropriate module (`components/`, `state/`, or `services/`)
3. Add dependencies to `Cargo.toml` with `optional = true` and feature gate

## Testing

### Test CLI
```bash
cargo test
```

### Test GUI components
```bash
cargo test --features gui
```

## Key Design Decisions

1. **Single Repository**: CLI and GUI share the same repo for easier maintenance and code reuse
2. **Library Pattern**: Core logic is in `lib.rs`, binaries are thin wrappers
3. **Feature Gates**: GUI dependencies are optional to keep CLI builds lightweight
4. **Shared State**: Business logic is identical between CLI and GUI
5. **Independent Releases**: While in the same repo, binaries can be released independently

## Next Steps

See [GUI_PLAN.md](GUI_PLAN.md) for the detailed implementation roadmap.

### Phase 1: Foundation (Current)
- [x] Set up dual binary structure
- [x] Create GUI scaffolding
- [ ] Implement basic Record Panel
- [ ] Set up file watching service

### Phase 2: Core Features
- [ ] Complete all 5 main panels
- [ ] Real-time status monitoring
- [ ] History management
- [ ] Recovery interface

### Phase 3: Polish
- [ ] Theme system
- [ ] System tray
- [ ] Notifications
- [ ] Keyboard shortcuts

## Resources

- [GPUI Documentation](https://www.gpui.rs/)
- [GPUI GitHub](https://github.com/zed-industries/zed)
- [GUI Plan](GUI_PLAN.md) - Complete implementation specification
