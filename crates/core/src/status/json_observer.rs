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

    pub fn write_processing_status_v2(
        &self,
        session_id: &str,
        message: &str,
        step: Option<u8>,
        total_steps: Option<u8>,
        processing_type: Option<&str>,
        file_size_bytes: Option<u64>,
        duration_secs: Option<u64>,
    ) -> Result<()> {
        let status_file = self.get_status_file(session_id);
        let mut processing_status = serde_json::json!({
            "session_id": session_id,
            "status": "processing",
            "message": message,
        });

        if let Some(step) = step {
            processing_status["step"] = serde_json::json!(step);
        }
        if let Some(total_steps) = total_steps {
            processing_status["total_steps"] = serde_json::json!(total_steps);
        }
        if let Some(processing_type) = processing_type {
            processing_status["processing_type"] = serde_json::json!(processing_type);
        }
        if let Some(file_size) = file_size_bytes {
            processing_status["file_size_bytes"] = serde_json::json!(file_size);
        }
        if let Some(duration) = duration_secs {
            processing_status["duration_secs"] = serde_json::json!(duration);
        }

        let json = serde_json::to_string_pretty(&processing_status)?;
        fs::write(&status_file, json)?;
        Ok(())
    }

    /// Update FFmpeg progress in the status file
    pub fn update_ffmpeg_progress(
        &self,
        session_id: &str,
        ffmpeg_progress: u8,
        processing_speed: Option<String>,
    ) -> Result<()> {
        let status_file = self.get_status_file(session_id);

        // Read existing status
        if let Ok(content) = fs::read_to_string(&status_file) {
            if let Ok(mut status) = serde_json::from_str::<serde_json::Value>(&content) {
                // Update FFmpeg progress fields
                status["ffmpeg_progress"] = serde_json::json!(ffmpeg_progress);
                if let Some(speed) = processing_speed {
                    status["processing_speed"] = serde_json::json!(speed);
                }

                // Write back
                let json = serde_json::to_string_pretty(&status)?;
                fs::write(&status_file, json)?;
                return Ok(());
            }
        }

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
