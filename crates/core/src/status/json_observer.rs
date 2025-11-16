use super::observer::{RecordingResult, RecordingStatus, StatusObserver};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

pub struct JsonFileObserver {
    status_dir: PathBuf,
}

impl JsonFileObserver {
    pub fn new(status_dir: PathBuf) -> Self {
        Self { status_dir }
    }

    fn get_status_file(&self, session_id: &str) -> PathBuf {
        self.status_dir.join(format!("{}.json", session_id))
    }

    pub fn write_processing_status(&self, session_id: &str, message: &str) -> Result<()> {
        let status_file = self.get_status_file(session_id);
        let processing_status = serde_json::json!({
            "session_id": session_id,
            "status": "processing",
            "message": message,
        });
        let json = serde_json::to_string_pretty(&processing_status)?;
        fs::write(&status_file, json)?;
        Ok(())
    }
}

impl StatusObserver for JsonFileObserver {
    fn on_progress(&self, status: RecordingStatus) -> Result<()> {
        let status_file = self.get_status_file(&status.session_id);
        let json = serde_json::to_string_pretty(&status)?;
        fs::write(&status_file, json)?;
        Ok(())
    }

    fn on_complete(&self, result: RecordingResult) -> Result<()> {
        let status_file = self.get_status_file(&result.session_id);
        let json = serde_json::to_string_pretty(&result)?;
        fs::write(&status_file, json)?;
        Ok(())
    }
}
