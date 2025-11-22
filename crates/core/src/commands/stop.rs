use anyhow::Context;
use serde_json::json;
use std::path::PathBuf;

use crate::config::RecorderConfig;
use crate::error::Result;
use crate::status::RecordingStatus;

/// Execute the stop command
pub async fn execute(session_id: Option<String>, config: RecorderConfig) -> Result<()> {
    config.ensure_directories()?;

    // Find target session
    let target_session = if let Some(id) = session_id {
        id
    } else {
        // Find latest active recording from status files
        find_active_session(&config.status_dir)?
    };

    // Create stop signal
    let signal_path = config.signals_dir.join(format!("{}.stop", target_session));
    std::fs::write(&signal_path, "")
        .with_context(|| format!("Failed to create stop signal file: {:?}", signal_path))?;

    // Return success
    let result = json!({
        "status": "success",
        "data": {
            "session_id": target_session,
            "message": "Stop signal sent successfully"
        }
    });

    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

fn find_active_session(status_dir: &PathBuf) -> Result<String> {
    use std::fs;

    // Read all files in status directory
    let entries = fs::read_dir(status_dir).context("Failed to read status directory")?;

    let mut active_sessions = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Read and parse the status file
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(status) = serde_json::from_str::<RecordingStatus>(&content) {
                // Check if status is "recording"
                if status.status == "recording" {
                    active_sessions.push((status.session_id, path));
                }
            }
        }
    }

    // Return the most recent active session
    if active_sessions.is_empty() {
        return Err(anyhow::anyhow!("No active recording sessions found").into());
    }

    // Sort by session_id (which contains timestamp) and get the latest
    active_sessions.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(active_sessions[0].0.clone())
}
