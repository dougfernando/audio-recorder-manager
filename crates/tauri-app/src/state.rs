//! Application state management

use audio_recorder_manager_core::audio_monitor::windows_monitor::AudioLevelMonitor;
use std::sync::Mutex;

/// Global application state shared across all Tauri commands
pub struct AppState {
    /// Active recording session IDs being managed by the app
    pub active_sessions: Mutex<Vec<String>>,
    /// Audio level monitor for real-time audio level detection
    pub audio_monitor: Mutex<Option<AudioLevelMonitor>>,
}

impl AppState {
    /// Create a new application state with empty sessions and no monitor
    pub fn new() -> Self {
        Self {
            active_sessions: Mutex::new(Vec::new()),
            audio_monitor: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
