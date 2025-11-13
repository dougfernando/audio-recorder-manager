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

    /// GUI-specific configuration (reserved for future use)
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub duration: RecordingDuration,
    pub format: AudioFormat,
    pub quality: RecordingQuality,
    #[allow(dead_code)]
    pub is_manual: bool,
    // Progress tracking fields
    pub elapsed: u64,
    pub progress: u8,
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames_captured: u64,
    pub has_audio: bool,
    pub status: String,
}

/// GUI-specific configuration
#[derive(Debug, Clone)]
pub struct GuiConfig {
    #[allow(dead_code)]
    pub theme: Theme,
    #[allow(dead_code)]
    pub window_width: u32,
    #[allow(dead_code)]
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
#[allow(dead_code)]
pub enum Theme {
    Light,
    Dark,
    System,
}
