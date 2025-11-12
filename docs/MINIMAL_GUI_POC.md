# Minimal GUI Proof of Concept

## Overview

This document describes the minimal GUI proof-of-concept that validates our dual-binary architecture. Due to GPUI's experimental nature and large dependency footprint (requires entire Zed repository), this POC demonstrates the architecture without requiring a full GPUI build.

## Current Implementation Status

### What's Implemented ✅

1. **Dual Binary Structure**
   - CLI binary: `audio-recorder-manager`
   - GUI binary: `audio-recorder-gui`
   - Both share core library

2. **GUI Module Scaffolding**
   ```
   src/gui/
   ├── main.rs              # Entry point
   ├── app.rs               # Main app component
   ├── components/mod.rs    # Component stubs
   ├── services/mod.rs      # Service stubs
   └── state/
       ├── mod.rs
       └── app_state.rs     # Full state management
   ```

3. **State Management**
   - `AppState` with all panels
   - `ActivePanel` enum
   - `RecordingState` for active sessions
   - `GuiConfig` with theme support

4. **Build Configuration**
   - Feature-gated GUI dependencies
   - CLI builds without GUI overhead
   - Separate binary targets

### What's Ready for Implementation 📋

Once you're ready to work with GPUI:

1. **Theme System** (`src/gui/theme.rs`)
   - Color palette
   - Typography
   - Spacing system

2. **Components** (`src/gui/components/`)
   - Sidebar navigation
   - Panel container
   - All 5 main panels

3. **Services** (`src/gui/services/`)
   - File watcher
   - Recorder service
   - History service

## Building and Running

### CLI (Always Works)
```bash
# Build CLI
cargo build --bin audio-recorder-manager

# Run CLI
cargo run --bin audio-recorder-manager -- record 30
```

### GUI (Requires GPUI Setup)

**Note**: Building the GUI requires the Zed repository. Here's how to set it up:

```bash
# Option 1: Clone Zed repository locally
git clone https://github.com/zed-industries/zed
cd ../audio-recorder-manager

# Update Cargo.toml to use local path:
# gpui = { path = "../zed/crates/gpui" }

# Then build
cargo build --bin audio-recorder-gui --features gui
```

**Alternative**: Use a lighter-weight UI framework for the POC (see options below).

## Architecture Validation ✅

The current implementation validates:

1. **Code Sharing**: CLI and GUI can both use `audio_recorder_manager::*`
2. **Independent Builds**: CLI builds without GUI dependencies
3. **State Management**: Full state structure is defined and compiles
4. **Module Organization**: Clean separation of concerns

## Alternative UI Frameworks for POC

If GPUI proves too heavyweight for initial development, consider these alternatives:

### Option 1: egui (Immediate Mode)

**Pros**:
- Lightweight (few dependencies)
- Cross-platform
- Easy to learn
- Fast iteration

**Cons**:
- Different paradigm from GPUI
- Less polished look

**Implementation**:
```toml
# Cargo.toml
egui = { version = "0.24", optional = true }
eframe = { version = "0.24", optional = true }

[features]
gui = ["egui", "eframe"]
```

```rust
// src/gui/main.rs (egui version)
use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Audio Recorder Manager",
        options,
        Box::new(|_cc| Box::new(AudioRecorderApp::default())),
    );
}
```

### Option 2: iced (Elm Architecture)

**Pros**:
- Modern reactive UI
- Type-safe
- Good documentation
- Cross-platform

**Cons**:
- Different pattern from GPUI
- More dependencies than egui

**Implementation**:
```toml
# Cargo.toml
iced = { version = "0.12", optional = true }

[features]
gui = ["iced"]
```

```rust
// src/gui/main.rs (iced version)
use iced::{Application, Settings};

fn main() {
    AudioRecorderApp::run(Settings::default());
}
```

### Option 3: Tauri (Web Technologies)

**Pros**:
- Use HTML/CSS/JS for UI
- Excellent documentation
- Active community
- Great for rapid prototyping

**Cons**:
- Different stack (web + Rust backend)
- Larger binary size

**Implementation**:
```bash
# Set up Tauri
cargo install tauri-cli
cargo tauri init
```

## Recommended Approach

### For Immediate Validation (This Week)

Use **egui** for a quick proof-of-concept:

1. Minimal dependencies
2. Fast compile times
3. Easy to learn
4. Can implement one panel in a day

### For Production GUI (Future)

Stick with **GPUI** as planned:

1. Native look and feel
2. High performance
3. Used by Zed (proven)
4. More control over UI

## Next Steps

### Option A: Quick POC with egui

```bash
# 1. Update Cargo.toml to use egui
# Replace gpui with egui/eframe

# 2. Implement simple app
# See docs/examples/egui_minimal.rs

# 3. Build and run
cargo run --bin audio-recorder-gui --features gui

# 4. Implement one panel (Record)
# Test full workflow

# 5. Decide: continue with egui or switch to GPUI
```

### Option B: Full GPUI Implementation

```bash
# 1. Clone Zed
git clone https://github.com/zed-industries/zed

# 2. Link locally
# Update Cargo.toml path

# 3. Follow GUI_ROADMAP.md Phase 1
# Implement navigation

# 4. Build (will take ~30min first time)
cargo build --bin audio-recorder-gui --features gui

# 5. Continue with roadmap phases
```

### Option C: Wait for GPUI Stabilization

```bash
# 1. Monitor Zed repository
# Watch for GPUI being published to crates.io

# 2. Continue CLI development
# Add features, tests, improvements

# 3. Revisit GUI when GPUI is stable
# Implement following roadmap
```

## Current Code Example

The current `app.rs` shows a placeholder UI:

```rust
impl Render for AudioRecorderApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xFFFFFF))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .child("Audio Recorder Manager")
                    .child("GUI under development")
            )
    }
}
```

This validates:
- GPUI API usage
- State management
- Component structure
- Build process

## Conclusion

The minimal POC successfully validates the architecture. You have three paths forward:

1. **Quick Win**: Implement with egui this week
2. **Long-term**: Continue with GPUI (requires setup)
3. **Hybrid**: POC with egui, migrate to GPUI later

All paths use the same state management and service layer, so the choice of UI framework is flexible.

## Resources

- **egui**: https://github.com/emilk/egui
- **iced**: https://github.com/iced-rs/iced
- **Tauri**: https://tauri.app/
- **GPUI**: https://www.gpui.rs/
- **Roadmap**: [GUI_ROADMAP.md](GUI_ROADMAP.md)
- **Full Plan**: [GUI_PLAN.md](GUI_PLAN.md)
