// Global application state

use audio_recorder_manager::{AudioFormat, RecorderConfig, RecordingDuration, RecordingQuality};

/// Main application state
pub struct AppState {
    /// Application configuration
    pub config: RecorderConfig,

    /// Currently active panel
    pub active_panel: ActivePanel,

    /// Recording state (if recording is active)
    pub recording_state: Option<RecordingState>,

    /// GUI-specific configuration
    pub gui_config: GuiConfig,
}

impl AppState {
    pub fn new(config: RecorderConfig) -> Self {
        Self {
            config,
            active_panel: ActivePanel::Record,
            recording_state: None,
            gui_config: GuiConfig::default(),
        }
    }
}

/// Which panel is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Record,
    Monitor,
    History,
    Recovery,
    Settings,
}

/// State for an active recording session
#[derive(Debug, Clone)]
pub struct RecordingState {
    pub session_id: String,
    pub filename: String,
    pub duration: RecordingDuration,
    pub format: AudioFormat,
    pub quality: RecordingQuality,
    pub is_manual: bool,
}

/// GUI-specific configuration
#[derive(Debug, Clone)]
pub struct GuiConfig {
    pub theme: Theme,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            window_width: 1024,
            window_height: 768,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}
