use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RecorderConfig {
    pub recordings_dir: PathBuf,
    pub status_dir: PathBuf,
    pub signals_dir: PathBuf,
    pub max_manual_duration_secs: u64,
    pub status_update_interval: Duration,
    pub buffer_duration_reftimes: i64,
    pub file_write_delay_ms: u64,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            recordings_dir: PathBuf::from("storage/recordings"),
            status_dir: PathBuf::from("storage/status"),
            signals_dir: PathBuf::from("storage/signals"),
            max_manual_duration_secs: 7200, // 2 hours
            status_update_interval: Duration::from_secs(1),
            buffer_duration_reftimes: 10_000_000, // 1 second in WASAPI reference time units
            file_write_delay_ms: 500,
        }
    }
}

impl RecorderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.recordings_dir)?;
        std::fs::create_dir_all(&self.status_dir)?;
        std::fs::create_dir_all(&self.signals_dir)?;
        Ok(())
    }
}
