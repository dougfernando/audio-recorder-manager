# macOS Migration Analysis

## Executive Summary

**Difficulty Level: MEDIUM to MEDIUM-HIGH**

The Audio Recorder Manager can be migrated to macOS with **moderate effort**. The application already has a non-Windows code path using the cross-platform `cpal` library, which supports macOS. However, **the flagship dual-channel recording feature (system audio + microphone simultaneously) will be significantly more challenging** on macOS due to platform restrictions.

---

## Current Architecture Overview

### Platform-Specific Components

The application has two distinct recording paths:

#### Windows Path (Current Production)
- **System Audio Capture**: WASAPI Loopback API (`wasapi_loopback.rs`)
- **Microphone Capture**: WASAPI Microphone API (`wasapi_microphone.rs`)
- **Audio Level Monitoring**: WASAPI-based real-time monitoring (`audio_monitor.rs`)
- **Key Feature**: Simultaneous dual-channel recording (system + mic)
- **Dependencies**: Windows-specific crates (`windows = "0.58"`)

#### Non-Windows Path (Basic Implementation Exists)
- **Audio Capture**: CPAL library (`recorder.rs`, `devices.rs`)
- **Status**: Functional but basic - single input device only
- **Limitation**: Cannot capture system audio (loopback) on macOS without additional setup

---

## What Already Works on macOS

### ✅ Functional Components (No Changes Needed)

1. **Core Business Logic**
   - All command modules (`record`, `stop`, `recover`, `status`)
   - Session management and state tracking
   - Configuration system with workspace detection
   - JSON status file writing and monitoring

2. **Audio Processing (FFmpeg-based)**
   - WAV to M4A conversion
   - Audio merging and resampling
   - Smart dual-mono channel processing
   - FFmpeg is cross-platform (already listed in README for macOS)

3. **Transcription Features**
   - HTTP API integration
   - Configuration management
   - All transcription logic

4. **File System Operations**
   - Storage directory management
   - Recording file management
   - Status and signal file handling

5. **Tauri Desktop App**
   - Svelte UI (100% cross-platform)
   - Most Tauri backend code (async command handlers)
   - File watching and notifications
   - System tray integration (Tauri 2.x supports macOS)

6. **CPAL-based Recording**
   - Basic microphone recording works
   - Device enumeration
   - WAV file writing
   - Quality presets

---

## What Needs to Be Implemented/Changed

### 🔶 Medium Difficulty: Basic macOS Support

#### 1. System Audio Capture (Loopback) - **MEDIUM-HIGH**

**Challenge**: macOS does **not** provide a native API like Windows WASAPI Loopback for capturing system audio.

**Solution Options**:

**Option A: BlackHole Virtual Audio Driver** (Recommended for MVP)
- **Effort**: LOW-MEDIUM
- **Description**: Use BlackHole (open-source virtual audio driver)
- **User Setup Required**: Install BlackHole, create Multi-Output Device in Audio MIDI Setup
- **Pros**:
  - Reliable, widely used in audio apps
  - Free and open-source
  - Works well for the use case
- **Cons**:
  - Requires user configuration (one-time setup)
  - Not fully automatic like Windows
- **Implementation**:
  - Detect BlackHole device via CPAL
  - Provide setup instructions in docs/UI
  - Guide users through Audio MIDI Setup

**Option B: Core Audio Screen Capture API** (macOS 13+ Sonoma)
- **Effort**: HIGH
- **Description**: Use `ScreenCaptureKit` API for audio capture
- **Pros**:
  - Native Apple API (no third-party driver)
  - Can capture specific app audio
  - User grants permission via system dialog
- **Cons**:
  - Requires macOS 13+ Sonoma (limits compatibility)
  - Requires Objective-C/Swift interop (via `objc` crate or Swift bridge)
  - More complex implementation
  - Need to request screen recording permission
- **Implementation Complexity**:
  - Write Objective-C/Swift wrapper
  - Handle Core Audio buffer format conversions
  - Manage permission dialogs and user consent

**Option C: Soundflower (Legacy)**
- **Effort**: LOW-MEDIUM
- **Status**: Discontinued, use BlackHole instead
- **Not Recommended**: BlackHole is the modern replacement

**Recommendation**: Start with **Option A (BlackHole)** for initial macOS support, then potentially add **Option B (ScreenCaptureKit)** as an advanced feature for users on macOS 13+.

#### 2. Audio Level Monitoring - **LOW-MEDIUM**

**Current State**: `audio_monitor.rs` is Windows-only (uses WASAPI)

**Solution**:
- Implement macOS version using Core Audio APIs via CPAL
- Already have atomics infrastructure in place
- Similar pattern to existing WASAPI implementation
- **Estimated Effort**: 2-3 days
- **Files to Create**:
  - `crates/core/src/audio_monitor.rs` - Add `#[cfg(target_os = "macos")]` module
  - Use CPAL to create input streams for monitoring
  - Calculate RMS levels same way as Windows version

#### 3. Device Detection - **MINIMAL**

**Current State**: `devices.rs` already uses CPAL (cross-platform)

**Changes Needed**:
- Test and verify on macOS
- May need device name mapping adjustments
- Handle BlackHole device detection specially
- **Estimated Effort**: 1-2 days

#### 4. Tauri App Platform-Specific Code - **LOW**

**Current Issues**:
- `main.rs` imports Windows-only `splash_screen` module
- Uses `#[cfg(windows)]` for command spawning flags
- `AudioLevelMonitor` import is Windows-only

**Changes Needed**:
```rust
// Current (line 4-5 in main.rs):
#[cfg(windows)]
mod splash_screen;

// Need to add:
#[cfg(target_os = "macos")]
mod macos_audio;  // macOS-specific setup/monitoring

// Current (line 8):
audio_monitor::windows_monitor::AudioLevelMonitor,

// Change to:
#[cfg(windows)]
use audio_recorder_manager_core::audio_monitor::windows_monitor::AudioLevelMonitor;
#[cfg(target_os = "macos")]
use audio_recorder_manager_core::audio_monitor::macos_monitor::AudioLevelMonitor;
```

**Estimated Effort**: 2-3 days

#### 5. Build and Distribution - **LOW-MEDIUM**

**Current State**:
- Only Windows build tested
- README mentions macOS but no binaries

**Changes Needed**:
- Add macOS target to CI/CD if exists
- Test build on macOS (Intel + Apple Silicon)
- Create `.dmg` installer via Tauri bundler
- Update README with macOS-specific instructions
- **Estimated Effort**: 2-3 days (assuming CI/CD already exists)

#### 6. Documentation and User Onboarding - **LOW**

**New Docs Needed**:
- macOS setup guide (BlackHole installation)
- Audio MIDI Setup configuration screenshots
- Troubleshooting guide for common macOS issues
- Permission handling (microphone access)
- **Estimated Effort**: 2-3 days

---

## File-by-File Change Summary

### Files Requiring Changes

| File | Change Required | Effort |
|------|----------------|--------|
| `crates/core/src/audio_monitor.rs` | Add macOS implementation module | Medium |
| `crates/core/src/devices.rs` | Test and minor adjustments for macOS | Low |
| `crates/tauri-app/src/main.rs` | Conditional compilation for AudioLevelMonitor | Low |
| `crates/tauri-app/Cargo.toml` | May need macOS-specific dependencies | Low |
| `crates/core/Cargo.toml` | May need Core Audio bindings (if using ScreenCaptureKit) | Medium |
| `docs/macos-setup.md` | **NEW** - User setup guide | Low |
| `docs/architecture.md` | Update with macOS platform notes | Low |
| `README.md` | Add macOS installation section | Low |
| `.github/workflows/*` | Add macOS build target (if CI/CD exists) | Low-Medium |

### Files Needing Minimal/No Changes

| File | Status |
|------|--------|
| `crates/core/src/commands/*.rs` | No changes (already has `#[cfg(not(windows))]` paths) |
| `crates/core/src/recorder.rs` | No changes (CPAL implementation exists) |
| `crates/core/src/wasapi_*.rs` | No changes (Windows-only, ignored on macOS) |
| `crates/core/src/transcription/*` | No changes (cross-platform) |
| `crates/tauri-app/ui/**/*` | No changes (Svelte is cross-platform) |
| `crates/cli/src/main.rs` | No changes (delegates to core) |

---

## Dependencies Analysis

### Current Dependencies (Cross-Platform)

✅ **Already Compatible with macOS**:
- `tokio` - Async runtime
- `serde`, `serde_json` - Serialization
- `anyhow`, `thiserror` - Error handling
- `chrono` - Date/time
- `hound` - WAV file I/O
- `cpal` - Cross-platform audio (supports macOS via Core Audio)
- `reqwest` - HTTP client (transcription API)
- `dirs` - User directories
- `tauri` - Desktop app framework (fully supports macOS)
- `notify` - File watching

### Platform-Specific Dependencies

❌ **Windows-Only** (Ignored on macOS):
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [...] }
```

✅ **Potential New Dependencies for macOS**:

**If using ScreenCaptureKit (Option B)**:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
# Option 1: Pure Rust bindings (if available)
screencapturekit = "0.1"  # (hypothetical, may not exist)

# Option 2: Objective-C bridge
objc = "0.2"
cocoa = "0.25"
core-foundation = "0.9"
core-audio-sys = "0.2"
```

**If using BlackHole (Option A)**:
- No new dependencies needed (uses CPAL to detect virtual device)

---

## Dual-Channel Recording: macOS Limitations

### Windows Approach (Current)
```
┌─────────────────────────────────────────────────────────────┐
│                    Windows WASAPI                           │
├─────────────────────────────────────────────────────────────┤
│  Loopback Recorder  →  system_audio.wav  (Left Channel)    │
│  Microphone Recorder → microphone.wav     (Right Channel)  │
│                                                             │
│  FFmpeg Merge → final_stereo.wav (Dual-mono: L=sys, R=mic) │
└─────────────────────────────────────────────────────────────┘
```

### macOS Approach (Proposed)

**With BlackHole**:
```
┌─────────────────────────────────────────────────────────────────┐
│                    macOS Core Audio + BlackHole                 │
├─────────────────────────────────────────────────────────────────┤
│  [USER SETUP REQUIRED]                                          │
│  1. Install BlackHole                                           │
│  2. Create Multi-Output Device (System + BlackHole)             │
│  3. Set Multi-Output as default output                          │
│                                                                 │
│  CPAL Recorder 1 → BlackHole input (system audio)              │
│  CPAL Recorder 2 → Default microphone                          │
│                                                                 │
│  FFmpeg Merge → final_stereo.wav (Dual-mono: L=sys, R=mic)    │
└─────────────────────────────────────────────────────────────────┘
```

**Key Difference**:
- Windows: Fully automatic, OS-native API
- macOS: Requires one-time user setup with virtual audio driver

**User Experience Impact**:
- Windows users: Zero setup, works out of the box
- macOS users: 5-10 minute one-time setup following guided instructions

---

## Implementation Phases

### Phase 1: Basic Recording (1-2 weeks)
**Goal**: Microphone-only recording on macOS

- [x] Test existing CPAL code on macOS
- [ ] Implement macOS audio level monitoring
- [ ] Fix Tauri conditional compilation issues
- [ ] Test basic recording workflow (mic only)
- [ ] Create macOS build scripts

**Deliverable**: CLI and Tauri app can record microphone audio on macOS

### Phase 2: System Audio Capture (2-3 weeks)
**Goal**: Dual-channel recording with BlackHole

- [ ] Implement BlackHole device detection
- [ ] Create user setup guide with screenshots
- [ ] Test dual-channel recording workflow
- [ ] Implement setup validation (detect if BlackHole configured)
- [ ] Add UI guidance for unconfigured systems

**Deliverable**: Full feature parity with Windows (with user setup requirement)

### Phase 3: Polish and Distribution (1-2 weeks)
**Goal**: Production-ready macOS app

- [ ] Create DMG installer with Tauri bundler
- [ ] Add app signing (for Gatekeeper)
- [ ] Test on Intel and Apple Silicon Macs
- [ ] Update all documentation
- [ ] Add macOS CI/CD pipeline
- [ ] Release beta for testing

**Deliverable**: Signed, distributable macOS application

### Phase 4 (Optional): Advanced Features (3-4 weeks)
**Goal**: Native system audio capture (no virtual driver)

- [ ] Implement ScreenCaptureKit API wrapper
- [ ] Handle permission dialogs
- [ ] Test on macOS 13+ Sonoma
- [ ] Fallback to BlackHole on older macOS
- [ ] Document macOS version requirements

**Deliverable**: Fully native macOS experience on modern systems

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CPAL incompatibility with macOS | Low | High | CPAL is mature and well-tested on macOS |
| BlackHole device detection fails | Medium | Medium | Provide manual device selection in UI |
| Sample rate mismatches | Medium | Low | FFmpeg already handles resampling |
| Permission issues (microphone) | High | Medium | Implement permission request flow in Tauri |
| Apple Silicon build issues | Low | Medium | Test on both architectures early |

### User Experience Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Users confused by BlackHole setup | High | High | Create detailed guide with screenshots, video tutorial |
| Users skip setup, system audio fails | High | Medium | Detect configuration state, show in-app setup wizard |
| App not signed, Gatekeeper blocks | High | High | Implement app signing in build process |

---

## Effort Estimation

### Development Time (Full-Time Developer)

| Phase | Estimated Time | Complexity |
|-------|---------------|------------|
| Phase 1: Basic Recording | 1-2 weeks | Low-Medium |
| Phase 2: System Audio (BlackHole) | 2-3 weeks | Medium |
| Phase 3: Polish & Distribution | 1-2 weeks | Low-Medium |
| **Total (MVP)** | **4-7 weeks** | **Medium** |
| Phase 4: ScreenCaptureKit (Optional) | 3-4 weeks | High |

### Breakdown by Task

| Task | Time | Notes |
|------|------|-------|
| macOS audio monitoring implementation | 3-4 days | Similar to Windows WASAPI approach |
| BlackHole integration & testing | 5-7 days | Device detection, dual-channel logic |
| Tauri conditional compilation fixes | 2-3 days | Platform-specific imports |
| macOS build setup (Intel + ARM) | 2-3 days | Tauri bundler configuration |
| Documentation & setup guide | 3-4 days | Screenshots, troubleshooting |
| Testing & bug fixes | 5-7 days | End-to-end testing on real macOS devices |
| App signing & distribution | 2-3 days | Developer certificate, notarization |

---

## Recommended Approach

### Step-by-Step Migration Plan

1. **Start with Phase 1 (Basic Recording)**
   - Lowest risk, quickest win
   - Validates CPAL implementation on macOS
   - Builds confidence in cross-platform approach

2. **Implement BlackHole Support (Phase 2)**
   - Document setup process first (helps validate UX)
   - Test thoroughly before moving to Phase 3
   - Consider building a setup wizard in the Tauri app

3. **Polish and Ship (Phase 3)**
   - Get beta testers on macOS
   - Iterate based on feedback
   - Don't skip app signing (critical for distribution)

4. **Consider ScreenCaptureKit Later (Phase 4)**
   - Only if BlackHole UX is problematic
   - Nice-to-have, not essential for MVP
   - Can be added in a future release

---

## Alternative: Cross-Platform Audio Library

### Consideration: Switching from WASAPI to CPAL on All Platforms

**Pros**:
- Single code path for all platforms
- Reduced maintenance burden
- Simpler architecture

**Cons**:
- CPAL may not have loopback support on Windows
- Would lose current Windows dual-channel implementation
- WASAPI is more robust on Windows

**Verdict**: **NOT RECOMMENDED**. Keep Windows WASAPI implementation, add macOS-specific code alongside it.

---

## Conclusion

### Migration Difficulty: **MEDIUM to MEDIUM-HIGH**

#### What Makes It Easier:
- ✅ Strong cross-platform foundation (CPAL, Tokio, FFmpeg)
- ✅ Existing non-Windows code path already in place
- ✅ Clear separation between platform-specific and shared code
- ✅ Tauri fully supports macOS

#### What Makes It Harder:
- ❌ System audio capture not built into macOS (requires workaround)
- ❌ User setup required for full feature parity
- ❌ Need to test on multiple macOS versions and architectures
- ❌ App signing and notarization for distribution

### Is It Worth It?

**YES**, if:
- You want to reach macOS users (large professional audio/meeting user base)
- You can accept a one-time setup requirement (BlackHole)
- You have 4-7 weeks for development and testing

**MAYBE WAIT**, if:
- You need 100% automatic system audio capture (wait for ScreenCaptureKit implementation)
- You don't have macOS hardware for testing
- Windows market is sufficient for now

---

## Next Steps

1. **Validate CPAL on macOS**
   - Clone repo on macOS
   - Test basic microphone recording
   - Verify device enumeration works

2. **Test BlackHole Integration**
   - Install BlackHole
   - Verify CPAL can detect it
   - Test dual-device recording

3. **Prototype Audio Monitoring**
   - Implement macOS module in `audio_monitor.rs`
   - Test real-time level detection

4. **Create Proof of Concept**
   - Get end-to-end recording working
   - Document any unexpected issues

5. **Make Go/No-Go Decision**
   - Based on PoC results
   - Estimate actual effort vs. planned
   - Decide on phasing approach

---

**Document Version**: 1.0
**Last Updated**: 2025-11-25
**Author**: AI Analysis Based on Codebase Review
