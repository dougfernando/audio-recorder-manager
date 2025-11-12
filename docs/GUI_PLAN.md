# Audio Recorder Manager - GUI Implementation Plan

## Executive Summary

This document outlines a comprehensive plan for building a desktop GUI application for the Audio Recorder Manager using Rust and the GPUI framework with the `gpui-component` library. The GUI will provide a modern, intuitive interface that maps all CLI functionality while adding visual monitoring, history management, and enhanced user experience.

## Technology Stack

### Core Framework
- **GPUI**: High-performance UI framework from the creators of Zed
- **gpui-component**: 60+ cross-platform desktop UI components
- **Architecture**: Declarative component-based model with RenderOnce pattern

### Integration Points
- Reuse existing Rust modules (commands, recorder, config, domain)
- File system watching for status file monitoring
- Async runtime (Tokio) for background tasks
- Shared codebase with CLI for business logic

## Application Architecture

### Project Structure
```
audio-recorder-manager/
├── src/
│   ├── main.rs                    # CLI entry point (existing)
│   ├── gui/
│   │   ├── main.rs                # GUI entry point
│   │   ├── app.rs                 # Main application state
│   │   ├── theme.rs               # Theme configuration
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── record_panel.rs    # Recording controls
│   │   │   ├── status_monitor.rs  # Real-time status display
│   │   │   ├── history_panel.rs   # Recording history viewer
│   │   │   ├── settings_panel.rs  # Configuration panel
│   │   │   ├── recovery_panel.rs  # Recovery interface
│   │   │   └── device_selector.rs # Audio device selection
│   │   ├── state/
│   │   │   ├── mod.rs
│   │   │   ├── app_state.rs       # Global application state
│   │   │   ├── recording_state.rs # Active recording state
│   │   │   └── history_state.rs   # Recording history state
│   │   └── services/
│   │       ├── mod.rs
│   │       ├── file_watcher.rs    # Watch status files
│   │       ├── recorder_service.rs # Wrap CLI commands
│   │       └── history_service.rs  # Manage recording history
│   ├── commands/                   # Existing CLI commands
│   ├── recorder.rs                 # Existing recorder logic
│   ├── config.rs                   # Existing config
│   └── domain.rs                   # Existing domain types
├── Cargo.toml
└── docs/
    └── GUI_PLAN.md                # This document
```

### Dual Entry Points
```toml
# Cargo.toml
[[bin]]
name = "audio-recorder-manager"
path = "src/main.rs"

[[bin]]
name = "audio-recorder-gui"
path = "src/gui/main.rs"
```

## UI Design & Layout

### Main Window Layout (Dock-based)

```
┌─────────────────────────────────────────────────────────────┐
│  Audio Recorder Manager                          [_][□][X]  │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌───────────────────────────────────────┐   │
│ │   Sidebar   │ │         Main Content Area             │   │
│ │             │ │                                       │   │
│ │ ○ Record    │ │   [Dynamic Panel based on selection]  │   │
│ │ ◉ Monitor   │ │                                       │   │
│ │ ○ History   │ │                                       │   │
│ │ ○ Recovery  │ │                                       │   │
│ │ ○ Settings  │ │                                       │   │
│ │             │ │                                       │   │
│ │             │ │                                       │   │
│ │             │ │                                       │   │
│ └─────────────┘ └───────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 1. Record Panel (Main View)

**Purpose**: Initiate and control new recordings

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│  Start New Recording                                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Duration:  [30] seconds  ○ Fixed  ○ Manual (-1)       │
│                                                         │
│  Format:    ○ WAV  ○ M4A                               │
│                                                         │
│  Quality:   ┌────────────────────────────┐              │
│             │ ○ Quick (16kHz Mono)       │              │
│             │ ○ Standard (44.1kHz Stereo)│              │
│             │ ◉ Professional (48kHz)     │  <-- Default │
│             │ ○ High (96kHz Stereo)      │              │
│             └────────────────────────────┘              │
│                                                         │
│  Device:    [System Audio + Microphone ▼]              │
│             └─ Dual-channel (Windows only)             │
│                                                         │
│  Output:    [storage/recordings/        📁]            │
│                                                         │
│             ┌──────────────────────┐                    │
│             │  🔴 START RECORDING  │  <-- Large button  │
│             └──────────────────────┘                    │
│                                                         │
│  Preview: recording_20250111_153045.wav (~11 MB/min)   │
└─────────────────────────────────────────────────────────┘
```

**Components Used**:
- Input fields (number input for duration)
- Radio buttons for format/quality selection
- Dropdown for device selection
- File picker button
- Large primary button for recording start
- Preview text label

**Interaction Flow**:
1. User configures recording parameters
2. Preview updates in real-time showing filename and estimated size
3. Click "START RECORDING" → transitions to Monitor panel
4. Recording starts via `commands::record::execute()`

### 2. Monitor Panel (Active Recording View)

**Purpose**: Display real-time recording status and controls

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│  Recording in Progress                                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Session: rec-20250111_153045                          │
│  File: recording_20250111_153045.wav                   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ Progress:  [████████████░░░░░░░░░░░] 67%        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  Time: 20s / 30s                                       │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  System Audio (Loopback)                        │   │
│  │  🔊 [████████████░░░░] 1,440 frames             │   │
│  │  Status: Audio Detected                         │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Microphone                                     │   │
│  │  🎤 [███████████░░░░░] 1,425 frames             │   │
│  │  Status: Audio Detected                         │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  Sample Rate: 48,000 Hz | Channels: 2 (Stereo)        │
│  Quality: Professional                                 │
│                                                         │
│             ┌──────────────────────┐                    │
│             │   ⏹️  STOP RECORDING  │                    │
│             └──────────────────────┘                    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Components Used**:
- Progress bar (main recording progress)
- Level meters (visual audio level indicators)
- Status badges ("Audio Detected" / "Silent")
- Real-time text updates (frames, time)
- Stop button (secondary danger style)

**Real-time Updates**:
- File watcher on `storage/status/{session_id}.json`
- Parse `RecordingStatus` updates every second
- Update progress bar, meters, frame counts
- Visual feedback for audio detection

**State Transitions**:
- Recording completes → Show completion dialog → Return to Record panel
- User clicks Stop → Call `commands::stop::execute()` → Show completion

### 3. History Panel

**Purpose**: Browse, manage, and play previous recordings

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│  Recording History                          🔍 [Search] │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Sort by: [Date ▼]  Filter: [All ▼]  Total: 47        │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ ┌─ recording_20250111_153045.wav ──────────────┐│   │
│  │ │ 📄 2.3 MB | WAV (PCM) | Professional         ││   │
│  │ │ Duration: 30s | Jan 11, 2025 3:30 PM        ││   │
│  │ │ Session: rec-20250111_153045                 ││   │
│  │ │ [▶️ Play] [📁 Show] [🗑️ Delete] [ℹ️ Details]  ││   │
│  │ └──────────────────────────────────────────────┘│   │
│  │                                                  │   │
│  │ ┌─ recording_20250111_120000.m4a ──────────────┐│   │
│  │ │ 📄 450 KB | M4A (AAC) | Professional         ││   │
│  │ │ Duration: 45s | Jan 11, 2025 12:00 PM       ││   │
│  │ │ Session: rec-20250111_120000                 ││   │
│  │ │ [▶️ Play] [📁 Show] [🗑️ Delete] [ℹ️ Details]  ││   │
│  │ └──────────────────────────────────────────────┘│   │
│  │                                                  │   │
│  │ ┌─ recording_20250110_204103.m4a ──────────────┐│   │
│  │ │ 📄 2.6 MB | M4A (AAC) | Standard             ││   │
│  │ │ Duration: 120s | Jan 10, 2025 8:41 PM       ││   │
│  │ │ Session: rec-20250110_204103                 ││   │
│  │ │ [▶️ Play] [📁 Show] [🗑️ Delete] [ℹ️ Details]  ││   │
│  │ └──────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  Page 1 of 5        [◀️] [1] [2] [3] [4] [5] [▶️]      │
└─────────────────────────────────────────────────────────┘
```

**Components Used**:
- Search input (text field)
- Dropdown filters (sort, format filter)
- Virtualized list/table for performance (200+ recordings)
- Card components for each recording
- Action buttons (icon buttons)
- Pagination controls

**Features**:
- **Search**: Filter by filename or session ID
- **Sort**: Date, size, duration, format, quality
- **Filter**: All, WAV only, M4A only
- **Actions**:
  - Play: Open system default audio player
  - Show: Open file location in explorer
  - Delete: Confirmation dialog → remove file
  - Details: Modal with full metadata

**Data Source**:
- Scan `storage/recordings/` directory
- Parse filenames and read file metadata
- Cache results for performance
- Auto-refresh on file changes

### 4. Recovery Panel

**Purpose**: Recover interrupted recordings

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│  Recovery Center                                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Incomplete Recordings Found: 2                        │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ ┌─ rec-20250111_140000 ────────────────────────┐│   │
│  │ │ ⚠️ Incomplete Recording                       ││   │
│  │ │ Found: rec-20250111_140000_loopback.wav      ││   │
│  │ │ Missing: rec-20250111_140000_mic.wav         ││   │
│  │ │ Date: Jan 11, 2025 2:00 PM                   ││   │
│  │ │                                               ││   │
│  │ │ Output format: ○ WAV  ○ M4A                  ││   │
│  │ │                                               ││   │
│  │ │ [🔧 Recover This Session]                     ││   │
│  │ └──────────────────────────────────────────────┘│   │
│  │                                                  │   │
│  │ ┌─ rec-20250111_130000 ────────────────────────┐│   │
│  │ │ ⚠️ Incomplete Recording                       ││   │
│  │ │ Found: rec-20250111_130000_loopback.wav      ││   │
│  │ │        rec-20250111_130000_mic.wav           ││   │
│  │ │ Date: Jan 11, 2025 1:00 PM                   ││   │
│  │ │                                               ││   │
│  │ │ Output format: ○ WAV  ○ M4A                  ││   │
│  │ │                                               ││   │
│  │ │ [🔧 Recover This Session]                     ││   │
│  │ └──────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  [🔧 Recover All as WAV] [🔧 Recover All as M4A]       │
│                                                         │
│  No incomplete recordings? Great! All your recordings  │
│  completed successfully.                                │
└─────────────────────────────────────────────────────────┘
```

**Components Used**:
- Warning badges/icons
- Card components for incomplete recordings
- Radio buttons for format selection
- Primary action buttons
- Empty state message

**Recovery Flow**:
1. Scan `storage/recordings/` for `*_loopback.wav` and `*_mic.wav`
2. Display incomplete sessions with found files
3. User selects format and clicks recover
4. Progress modal shows recovery process
5. Call `commands::recover::execute()`
6. Show success/error notification
7. Refresh list

### 5. Settings Panel

**Purpose**: Configure application preferences

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│  Settings                                               │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─ Directories ─────────────────────────────────────┐ │
│  │                                                    │ │
│  │  Recordings:  [storage/recordings/      📁]       │ │
│  │  Status:      [storage/status/          📁]       │ │
│  │  Signals:     [storage/signals/         📁]       │ │
│  │                                                    │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
│  ┌─ Recording Defaults ──────────────────────────────┐ │
│  │                                                    │ │
│  │  Default Duration:  [30] seconds                  │ │
│  │  Default Format:    ○ WAV  ◉ M4A                  │ │
│  │  Default Quality:   [Professional ▼]              │ │
│  │                                                    │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
│  ┌─ Advanced ────────────────────────────────────────┐ │
│  │                                                    │ │
│  │  Max Manual Duration:    [7200] seconds (2 hrs)   │ │
│  │  Status Update Interval: [1000] ms                │ │
│  │  File Write Delay:       [500] ms                 │ │
│  │                                                    │ │
│  │  ☑ Enable system notifications                    │ │
│  │  ☑ Auto-cleanup temporary files                   │ │
│  │  ☑ Minimize to system tray                        │ │
│  │                                                    │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
│  ┌─ Appearance ──────────────────────────────────────┐ │
│  │                                                    │ │
│  │  Theme:  ○ Light  ○ Dark  ◉ System                │ │
│  │  Accent Color: [Blue ▼]                           │ │
│  │                                                    │ │
│  └────────────────────────────────────────────────────┘ │
│                                                         │
│  [Save Settings] [Reset to Defaults]                   │
└─────────────────────────────────────────────────────────┘
```

**Components Used**:
- Collapsible sections (grouped settings)
- File picker inputs
- Number inputs
- Radio buttons, dropdowns
- Checkboxes
- Action buttons

**Settings Persistence**:
- Store in `config.json` or use platform-specific config
- Extends existing `RecorderConfig` struct
- GUI-specific settings (theme, notifications, etc.)

## State Management

### Application State Structure

```rust
// src/gui/state/app_state.rs
pub struct AppState {
    // Current view
    pub active_panel: ActivePanel,

    // Recording state
    pub recording_state: Option<RecordingState>,

    // History
    pub recordings: Vec<RecordingEntry>,
    pub history_filter: HistoryFilter,

    // Recovery
    pub incomplete_recordings: Vec<IncompleteRecording>,

    // Settings
    pub config: RecorderConfig,
    pub gui_config: GuiConfig,

    // UI state
    pub notifications: Vec<Notification>,
    pub is_loading: bool,
}

pub enum ActivePanel {
    Record,
    Monitor,
    History,
    Recovery,
    Settings,
}

// src/gui/state/recording_state.rs
pub struct RecordingState {
    pub session_id: String,
    pub filename: String,
    pub status: RecordingStatus,
    pub start_time: DateTime<Local>,
    pub is_manual: bool,
}

// src/gui/state/history_state.rs
pub struct RecordingEntry {
    pub filename: String,
    pub session_id: String,
    pub path: PathBuf,
    pub size: u64,
    pub format: AudioFormat,
    pub quality: String,
    pub duration: Option<u64>,
    pub created: DateTime<Local>,
}

pub struct HistoryFilter {
    pub search_query: String,
    pub sort_by: SortField,
    pub format_filter: Option<AudioFormat>,
    pub page: usize,
    pub page_size: usize,
}

// src/gui/state/gui_config.rs
pub struct GuiConfig {
    pub theme: Theme,
    pub accent_color: AccentColor,
    pub enable_notifications: bool,
    pub auto_cleanup: bool,
    pub minimize_to_tray: bool,
    pub window_size: (u32, u32),
    pub window_position: (i32, i32),
}
```

### State Updates & Events

```rust
// Event system for state changes
pub enum AppEvent {
    // Navigation
    NavigateTo(ActivePanel),

    // Recording
    StartRecording {
        duration: RecordingDuration,
        format: AudioFormat,
        quality: RecordingQuality,
    },
    StopRecording,
    RecordingStatusUpdated(RecordingStatus),
    RecordingCompleted(RecordingResult),

    // History
    RefreshHistory,
    SearchHistory(String),
    SortHistory(SortField),
    FilterHistory(Option<AudioFormat>),
    DeleteRecording(String),
    PlayRecording(PathBuf),

    // Recovery
    ScanForIncomplete,
    RecoverSession { session_id: String, format: AudioFormat },
    RecoverAll(AudioFormat),

    // Settings
    UpdateConfig(RecorderConfig),
    UpdateGuiConfig(GuiConfig),

    // Notifications
    ShowNotification(Notification),
    DismissNotification(usize),
}

// Handle events and update state
impl AppState {
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::NavigateTo(panel) => {
                self.active_panel = panel;
            }
            AppEvent::StartRecording { duration, format, quality } => {
                // Spawn recording task
                // Update state to show monitor panel
            }
            // ... handle other events
        }
    }
}
```

## Service Layer

### File Watcher Service

```rust
// src/gui/services/file_watcher.rs
use notify::{Watcher, RecursiveMode, Event};

pub struct FileWatcherService {
    watcher: RecommendedWatcher,
    status_dir: PathBuf,
    recordings_dir: PathBuf,
}

impl FileWatcherService {
    pub fn new(config: &RecorderConfig) -> Result<Self>;

    // Watch status files for active recording updates
    pub fn watch_status_file(
        &mut self,
        session_id: &str,
        callback: impl Fn(RecordingStatus) + Send + 'static,
    ) -> Result<()>;

    // Watch recordings directory for new files
    pub fn watch_recordings_dir(
        &mut self,
        callback: impl Fn() + Send + 'static,
    ) -> Result<()>;

    pub fn stop_watching(&mut self) -> Result<()>;
}
```

### Recorder Service

```rust
// src/gui/services/recorder_service.rs
pub struct RecorderService {
    config: RecorderConfig,
    active_session: Option<String>,
}

impl RecorderService {
    pub fn new(config: RecorderConfig) -> Self;

    // Start recording (wraps commands::record::execute)
    pub async fn start_recording(
        &mut self,
        duration: RecordingDuration,
        format: AudioFormat,
        quality: RecordingQuality,
    ) -> Result<RecordingSession>;

    // Stop recording (wraps commands::stop::execute)
    pub async fn stop_recording(&mut self, session_id: Option<String>) -> Result<()>;

    // Get current status
    pub fn get_status(&self, session_id: &str) -> Result<RecordingStatus>;

    // Check for active recordings
    pub fn has_active_recording(&self) -> bool;
}
```

### History Service

```rust
// src/gui/services/history_service.rs
pub struct HistoryService {
    recordings_dir: PathBuf,
    cache: Vec<RecordingEntry>,
    last_scan: Option<SystemTime>,
}

impl HistoryService {
    pub fn new(recordings_dir: PathBuf) -> Self;

    // Scan directory and build history
    pub fn scan(&mut self) -> Result<Vec<RecordingEntry>>;

    // Filter and sort recordings
    pub fn filter(
        &self,
        filter: &HistoryFilter,
    ) -> Vec<RecordingEntry>;

    // Get paginated results
    pub fn get_page(
        &self,
        filter: &HistoryFilter,
    ) -> (Vec<RecordingEntry>, usize); // (entries, total_pages)

    // Delete recording
    pub fn delete(&mut self, session_id: &str) -> Result<()>;

    // Get recording details
    pub fn get_details(&self, session_id: &str) -> Option<RecordingEntry>;
}
```

## UI Components Implementation

### Component Patterns

Using gpui-component's RenderOnce pattern:

```rust
// src/gui/components/record_panel.rs
use gpui::*;
use gpui_component::*;

pub struct RecordPanel {
    duration: u64,
    is_manual: bool,
    format: AudioFormat,
    quality: RecordingQuality,
}

impl RenderOnce for RecordPanel {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                h2("Start New Recording")
                    .text_2xl()
                    .font_semibold()
            )
            .child(
                // Duration input
                FormField::new("Duration")
                    .child(
                        Input::new()
                            .value(self.duration.to_string())
                            .on_change(|value, cx| {
                                // Update state
                            })
                    )
            )
            .child(
                // Format radio group
                RadioGroup::new("format")
                    .option("WAV", AudioFormat::Wav)
                    .option("M4A", AudioFormat::M4a)
                    .selected(self.format)
                    .on_change(|format, cx| {
                        // Update state
                    })
            )
            .child(
                // Start button
                Button::new("start_recording")
                    .label("START RECORDING")
                    .variant(ButtonVariant::Primary)
                    .size(ButtonSize::Large)
                    .on_click(|cx| {
                        // Trigger start recording event
                    })
            )
    }
}
```

### Key Components to Implement

1. **RecordPanel**: Recording configuration and start
2. **MonitorPanel**: Real-time status display
3. **AudioLevelMeter**: Custom component for audio visualization
4. **HistoryPanel**: Recording list with virtualization
5. **RecordingCard**: Individual recording display
6. **RecoveryPanel**: Incomplete recording management
7. **SettingsPanel**: Configuration interface
8. **NotificationToast**: System notifications
9. **ConfirmDialog**: Confirmation modals
10. **DeviceSelector**: Audio device picker

## Advanced Features

### 1. System Tray Integration

```rust
// Minimize to system tray
// Show recording status in tray icon
// Quick actions menu
use tray_icon::{TrayIcon, TrayIconBuilder};

pub struct TrayManager {
    icon: TrayIcon,
}

impl TrayManager {
    pub fn new() -> Self;
    pub fn update_status(&mut self, status: &str);
    pub fn show_menu(&mut self);
    pub fn restore_window(&mut self);
}
```

### 2. System Notifications

```rust
// Desktop notifications for events
use notify_rust::Notification;

pub fn notify_recording_started(filename: &str) {
    Notification::new()
        .summary("Recording Started")
        .body(&format!("Recording to {}", filename))
        .icon("microphone")
        .show()
        .ok();
}

pub fn notify_recording_complete(filename: &str, duration: u64) {
    Notification::new()
        .summary("Recording Complete")
        .body(&format!("{} ({} seconds)", filename, duration))
        .icon("check")
        .show()
        .ok();
}
```

### 3. Audio Playback Preview

```rust
// Quick playback in app (optional)
// Use rodio or similar for audio playback
use rodio::{Decoder, OutputStream, Sink};

pub struct AudioPlayer {
    sink: Sink,
    _stream: OutputStream,
}

impl AudioPlayer {
    pub fn new() -> Self;
    pub fn play(&mut self, path: &Path) -> Result<()>;
    pub fn pause(&mut self);
    pub fn stop(&mut self);
    pub fn seek(&mut self, position: Duration);
}
```

### 4. Drag & Drop Support

```rust
// Drag recordings out to desktop
// Drop files into recovery panel for manual recovery
impl HistoryPanel {
    fn handle_drag_start(&mut self, recording: &RecordingEntry) {
        // Create drag data with file path
    }
}

impl RecoveryPanel {
    fn handle_drop(&mut self, files: Vec<PathBuf>) {
        // Validate dropped files
        // Add to recovery queue
    }
}
```

### 5. Keyboard Shortcuts

```rust
// Global shortcuts
// Ctrl+R: Start recording
// Ctrl+S: Stop recording
// Ctrl+H: Go to history
// Ctrl+,: Settings
// Space: Play/pause in history
use gpui::KeyBinding;

pub fn register_shortcuts(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("ctrl-r", StartRecording, None),
        KeyBinding::new("ctrl-s", StopRecording, None),
        KeyBinding::new("ctrl-h", NavigateToHistory, None),
        // ... more shortcuts
    ]);
}
```

### 6. Export & Import Settings

```rust
// Export settings as JSON
// Import settings from file
pub fn export_settings(config: &GuiConfig) -> Result<String> {
    serde_json::to_string_pretty(config)
}

pub fn import_settings(json: &str) -> Result<GuiConfig> {
    serde_json::from_str(json)
}
```

## Theme System

### Theme Configuration

```rust
// src/gui/theme.rs
use gpui_component::theme::Theme;

pub struct AppTheme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_muted: Color,
}

impl AppTheme {
    pub fn light() -> Self {
        Self {
            primary: Color::rgb(0.2, 0.4, 0.8),
            // ... light theme colors
        }
    }

    pub fn dark() -> Self {
        Self {
            primary: Color::rgb(0.4, 0.6, 1.0),
            // ... dark theme colors
        }
    }

    pub fn from_system() -> Self {
        // Detect system preference
        if is_dark_mode() {
            Self::dark()
        } else {
            Self::light()
        }
    }
}
```

### Accent Colors

Support multiple accent colors:
- Blue (default)
- Purple
- Green
- Red
- Orange

## Error Handling & User Feedback

### Error Display

```rust
pub enum AppError {
    RecordingFailed(String),
    RecoveryFailed(String),
    FileNotFound(PathBuf),
    PermissionDenied(String),
    FFmpegNotFound,
    DeviceNotAvailable,
}

impl AppError {
    pub fn to_notification(&self) -> Notification {
        Notification {
            title: "Error".to_string(),
            message: self.to_string(),
            level: NotificationLevel::Error,
            duration: Duration::from_secs(5),
        }
    }
}
```

### Progress Indicators

- Recording progress bar
- Recovery progress modal
- Loading spinners for async operations
- Success/error toasts

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_filter() {
        let service = HistoryService::new(PathBuf::from("test"));
        let filter = HistoryFilter {
            search_query: "meeting".to_string(),
            ..Default::default()
        };
        let results = service.filter(&filter);
        // Assert results
    }

    #[test]
    fn test_incomplete_recording_detection() {
        // Test recovery panel logic
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_recording_workflow() {
    // Create GUI app state
    // Start recording
    // Monitor status updates
    // Stop recording
    // Verify file created
}
```

### Manual Testing Checklist

- [ ] Start/stop recordings in all quality modes
- [ ] Monitor real-time status updates
- [ ] Browse history with 100+ recordings
- [ ] Recover interrupted recordings
- [ ] Change settings and verify persistence
- [ ] Test keyboard shortcuts
- [ ] Verify system tray integration
- [ ] Test on Windows 10 and 11
- [ ] Test with different audio devices
- [ ] Test with/without FFmpeg installed

## Performance Considerations

### Optimization Strategies

1. **Virtualized Lists**: Use gpui-component's virtualized table for 200+ recordings
2. **Lazy Loading**: Load recording metadata on-demand
3. **Debouncing**: Debounce file watcher events (500ms)
4. **Caching**: Cache parsed status files
5. **Background Tasks**: Run file scans in background threads
6. **Efficient Re-renders**: Use GPUI's fine-grained reactivity

### Memory Management

- Limit status file history (keep last 100 updates)
- Cleanup old notifications (max 10)
- Release audio player resources when not in use
- Paginate history (50 items per page)

## Deployment & Distribution

### Build Configuration

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true

[package.metadata.bundle]
name = "Audio Recorder Manager"
identifier = "com.audiorecorder.manager"
icon = ["assets/icon.ico"]
version = "0.3.0"
resources = ["assets/*"]
```

### Platform-Specific Builds

**Windows**:
```bash
cargo build --release --bin audio-recorder-gui --target x86_64-pc-windows-gnu
# Create installer with WiX or Inno Setup
```

**macOS** (future):
```bash
cargo build --release --bin audio-recorder-gui --target x86_64-apple-darwin
cargo bundle
```

**Linux** (future):
```bash
cargo build --release --bin audio-recorder-gui
# Create .deb or .rpm package
```

### Installation

1. Standalone executable (current approach)
2. Optional: Windows installer with desktop shortcut
3. Optional: Auto-updater integration

## Migration Path from CLI

### Phase 1: Core GUI (v0.4.0)
- Basic record panel and monitor
- Simple history list
- Settings panel
- Theme support

### Phase 2: Enhanced Features (v0.5.0)
- Recovery panel
- System tray integration
- Notifications
- Audio preview

### Phase 3: Advanced Features (v0.6.0)
- Drag & drop
- Keyboard shortcuts
- Export/import settings
- Waveform visualization

### Phase 4: Cross-Platform (v0.7.0)
- macOS support
- Linux support
- Platform-specific optimizations

## Dependencies

### Required Crates

```toml
[dependencies]
# Existing dependencies
# ... (all current dependencies)

# GUI framework
gpui = "0.1"
gpui-component = "0.1"

# File watching
notify = "6.1"
notify-debouncer-full = "0.3"

# System tray (optional)
tray-icon = "0.14"

# Notifications (optional)
notify-rust = "4.10"

# Audio playback (optional)
rodio = "0.17"

[dev-dependencies]
# Testing
tempfile = "3.8"
```

## Documentation

### User Guide Topics

1. Installation and Setup
2. Recording Your First Audio
3. Understanding Quality Settings
4. Managing Your Recordings
5. Recovering Interrupted Recordings
6. Customizing Settings
7. Keyboard Shortcuts Reference
8. Troubleshooting

### Developer Documentation

1. Architecture Overview
2. Adding New Panels
3. Creating Custom Components
4. State Management Guide
5. Service Layer Design
6. Theme Customization
7. Contributing Guidelines

## Success Metrics

### Initial Release Goals

- [ ] Feature parity with CLI (all commands accessible)
- [ ] Real-time status updates with <100ms latency
- [ ] History panel handles 500+ recordings smoothly
- [ ] Application startup <2 seconds
- [ ] Memory usage <100MB idle
- [ ] All features tested on Windows 10/11
- [ ] Complete user documentation

## Future Enhancements

### Beyond Initial Release

1. **Audio Waveform Visualization**: Visual representation of recordings
2. **Batch Operations**: Process multiple recordings at once
3. **Cloud Sync**: Optional backup to cloud storage
4. **Transcription Integration**: Speech-to-text via Whisper
5. **Meeting Detection**: Auto-start when meeting software launches
6. **Scheduled Recordings**: Record at specific times
7. **Plugins System**: Extend functionality via plugins
8. **Mobile Companion**: iOS/Android remote control app

## Conclusion

This GUI implementation plan provides a comprehensive roadmap for building a modern, performant desktop application that enhances the audio-recorder-manager with visual monitoring, intuitive controls, and powerful management features. By leveraging GPUI and gpui-component, we can create a native-feeling application that maintains the performance characteristics of the Rust CLI while providing a superior user experience.

The phased approach allows for incremental development and early user feedback, ensuring each release delivers value while building toward a feature-complete desktop application.
