// Recorder service for managing recording operations from the GUI

use audio_recorder_manager::{
    commands, RecorderConfig, AudioFormat, RecordingDuration, RecordingQuality,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents the current recording state
#[derive(Debug, Clone)]
pub struct RecordingInfo {
    pub session_id: String,
    pub filename: String,
    pub duration: u64,
    pub elapsed: u64,
    pub progress: u8,
    pub is_recording: bool,
}

/// Service for managing audio recording operations
pub struct RecorderService {
    config: Arc<RecorderConfig>,
    current_recording: Arc<Mutex<Option<RecordingInfo>>>,
    tokio_runtime: Arc<tokio::runtime::Runtime>,
}

impl RecorderService {
    pub fn new(config: RecorderConfig, tokio_runtime: Arc<tokio::runtime::Runtime>) -> Self {
        Self {
            config: Arc::new(config),
            current_recording: Arc::new(Mutex::new(None)),
            tokio_runtime,
        }
    }

    /// Start a new recording session
    pub async fn start_recording(
        &self,
        duration: RecordingDuration,
        format: AudioFormat,
        quality: RecordingQuality,
    ) -> anyhow::Result<String> {
        // Check if already recording
        if self.current_recording.lock().await.is_some() {
            return Err(anyhow::anyhow!("A recording is already in progress"));
        }

        // Clone necessary data for the background task
        let config = (*self.config).clone();
        let current_recording = self.current_recording.clone();
        let session_id = format!("rec-{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let session_id_clone = session_id.clone();

        // Spawn the recording task on the dedicated Tokio runtime
        self.tokio_runtime.spawn(async move {
            match commands::record::execute(duration, format, quality, config).await {
                Ok(_) => {
                    let mut current = current_recording.lock().await;
                    *current = None;
                    log::info!("Recording completed successfully: {}", session_id_clone);
                }
                Err(e) => {
                    let mut current = current_recording.lock().await;
                    *current = None;
                    log::error!("Recording failed: {}: {}", session_id_clone, e);
                }
            }
        });

        // Set initial recording info immediately
        let filename = format!(
            "recording_{}.{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            format.extension()
        );
        {
            let mut current = self.current_recording.lock().await;
            *current = Some(RecordingInfo {
                session_id: session_id.clone(),
                filename,
                duration: duration.to_api_value() as u64,
                elapsed: 0,
                progress: 0,
                is_recording: true,
            });
        }

        Ok(session_id)
    }

    /// Stop the current recording session
    pub async fn stop_recording(&self) -> anyhow::Result<()> {
        // Get current session ID
        let session_id = {
            let current = self.current_recording.lock().await;
            current.as_ref()
                .map(|r| r.session_id.clone())
        };

        if let Some(id) = session_id {
            // Send stop signal
            let config = (*self.config).clone();
            commands::stop::execute(Some(id), config).await?;

            // Clear current recording
            let mut current = self.current_recording.lock().await;
            *current = None;

            Ok(())
        } else {
            Err(anyhow::anyhow!("No active recording to stop"))
        }
    }

    /// Get current recording status
    pub async fn get_current_recording(&self) -> Option<RecordingInfo> {
        let current = self.current_recording.lock().await;
        current.clone()
    }

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        let current = self.current_recording.lock().await;
        current.is_some()
    }

    /// Update recording progress (called periodically by GUI)
    pub async fn update_progress(&self) -> anyhow::Result<Option<RecordingInfo>> {
        let mut current = self.current_recording.lock().await;

        if let Some(ref mut info) = *current {
            // Read status from status file
            let status_file = self.config.status_dir.join(format!("{}.json", info.session_id));

            if status_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&status_file) {
                    if let Ok(status) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Update progress from status file
                        if let Some(elapsed) = status.get("elapsed").and_then(|v| v.as_u64()) {
                            info.elapsed = elapsed;
                        }
                        if let Some(progress) = status.get("progress").and_then(|v| v.as_u64()) {
                            info.progress = progress as u8;
                        }

                        // Check if completed
                        if let Some(status_str) = status.get("status").and_then(|v| v.as_str()) {
                            if status_str == "completed" {
                                info.is_recording = false;
                            }
                        }
                    }
                }
            }

            Ok(Some(info.clone()))
        } else {
            Ok(None)
        }
    }
}
