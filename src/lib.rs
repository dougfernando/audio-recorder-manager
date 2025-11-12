// Library module that exposes core functionality
// This allows both CLI and GUI binaries to use the same business logic

pub mod commands;
pub mod config;
pub mod devices;
pub mod domain;
pub mod error;
pub mod recorder;
pub mod status;

#[cfg(windows)]
pub mod wasapi_loopback;
#[cfg(windows)]
pub mod wasapi_microphone;

// Re-export commonly used types for convenience
pub use config::RecorderConfig;
pub use domain::{AudioFormat, RecordingDuration, RecordingSession, SessionId};
pub use error::{RecorderError, Result};
pub use recorder::RecordingQuality;
pub use status::{RecordingResult, RecordingStatus, StatusObserver};
