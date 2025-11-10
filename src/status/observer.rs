use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub session_id: String,
    pub filename: String,
    pub duration: u64,
    pub elapsed: u64,
    pub progress: u8,
    pub quality: String,
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames_captured: u64,
    pub has_audio: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResult {
    pub session_id: String,
    pub filename: String,
    pub duration: i64,
    pub file_size_mb: String,
    pub format: String,
    pub codec: String,
    pub message: String,
    pub status: String,
}

pub trait StatusObserver: Send + Sync {
    fn on_progress(&self, status: RecordingStatus) -> crate::error::Result<()>;
    fn on_complete(&self, result: RecordingResult) -> crate::error::Result<()>;
}
