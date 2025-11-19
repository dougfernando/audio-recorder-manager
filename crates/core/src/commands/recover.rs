use anyhow::Context;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::RecorderConfig;
use crate::domain::AudioFormat;
use crate::error::Result;
use crate::output::UserOutput;
use crate::recorder::{convert_wav_to_m4a, merge_audio_streams_smart, RecordingQuality};

/// Execute the recover command to finish interrupted recordings
pub async fn execute(
    session_id: Option<String>,
    target_format: Option<AudioFormat>,
    config: RecorderConfig,
) -> Result<()> {
    execute_with_output(session_id, target_format, config, UserOutput::new()).await
}

/// Execute the recover command with custom output
pub async fn execute_with_output(
    session_id: Option<String>,
    target_format: Option<AudioFormat>,
    config: RecorderConfig,
    output: UserOutput,
) -> Result<()> {
    config.ensure_directories()?;

    // Find incomplete recordings
    let incomplete = find_incomplete_recordings(&config.recordings_dir)?;

    if incomplete.is_empty() {
        let result = json!({
            "status": "success",
            "data": {
                "message": "No incomplete recordings found",
                "recovered": []
            }
        });
        println!("{}", serde_json::to_string(&result)?);
        return Ok(());
    }

    // Filter by session_id if provided
    let to_recover: Vec<_> = if let Some(ref id) = session_id {
        incomplete
            .into_iter()
            .filter(|r| &r.session_id == id)
            .collect()
    } else {
        incomplete
    };

    if to_recover.is_empty() {
        let result = json!({
            "status": "error",
            "error": format!("No incomplete recording found for session: {}", session_id.unwrap())
        });
        println!("{}", serde_json::to_string(&result)?);
        return Ok(());
    }

    let mut recovered = Vec::new();
    let mut errors = Vec::new();

    for recording in to_recover {
        output.prefixed("Recovery", &format!("Processing session: {}", recording.session_id));

        match recover_recording(&recording, target_format, &config, &output).await {
            Ok(output_path) => {
                let file_name = output_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                output.success(&format!("Successfully recovered: {}", file_name));
                recovered.push(json!({
                    "session_id": recording.session_id,
                    "output_file": file_name,
                    "output_path": output_path.to_string_lossy()
                }));
            }
            Err(e) => {
                output.error(&format!("Failed to recover {}: {}", recording.session_id, e));
                errors.push(json!({
                    "session_id": recording.session_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    let result = if errors.is_empty() {
        json!({
            "status": "success",
            "data": {
                "message": format!("Successfully recovered {} recording(s)", recovered.len()),
                "recovered": recovered
            }
        })
    } else {
        json!({
            "status": "partial",
            "data": {
                "message": format!("Recovered {} of {} recording(s)", recovered.len(), recovered.len() + errors.len()),
                "recovered": recovered,
                "errors": errors
            }
        })
    };

    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

#[derive(Debug)]
struct IncompleteRecording {
    session_id: String,
    loopback_file: Option<PathBuf>,
    mic_file: Option<PathBuf>,
    timestamp: String,
}

fn find_incomplete_recordings(recordings_dir: &PathBuf) -> Result<Vec<IncompleteRecording>> {
    if !recordings_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(recordings_dir)
        .context("Failed to read recordings directory")?;

    // Group files by session ID
    let mut sessions: HashMap<String, (Option<PathBuf>, Option<PathBuf>)> = HashMap::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Look for temporary WAV files
        if filename.ends_with("_loopback.wav") {
            let session_id = filename.trim_end_matches("_loopback.wav").to_string();
            let entry = sessions.entry(session_id).or_insert((None, None));
            entry.0 = Some(path);
        } else if filename.ends_with("_mic.wav") {
            let session_id = filename.trim_end_matches("_mic.wav").to_string();
            let entry = sessions.entry(session_id).or_insert((None, None));
            entry.1 = Some(path);
        }
    }

    // Convert to IncompleteRecording structs
    let mut incomplete = Vec::new();
    for (session_id, (loopback, mic)) in sessions {
        // Only include if at least one file exists
        if loopback.is_some() || mic.is_some() {
            // Extract timestamp from session_id (format: rec-YYYYMMDD_HHMMSS)
            let timestamp = session_id
                .strip_prefix("rec-")
                .unwrap_or(&session_id)
                .to_string();

            incomplete.push(IncompleteRecording {
                session_id,
                loopback_file: loopback,
                mic_file: mic,
                timestamp,
            });
        }
    }

    // Sort by timestamp (most recent first)
    incomplete.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(incomplete)
}

async fn recover_recording(
    recording: &IncompleteRecording,
    target_format: Option<AudioFormat>,
    config: &RecorderConfig,
    output: &UserOutput,
) -> Result<PathBuf> {
    // Determine output format (default to WAV if not specified)
    let format = target_format.unwrap_or(AudioFormat::Wav);

    // Use professional quality as default for recovery
    let quality = RecordingQuality::professional();

    // Create output filename based on timestamp
    let output_filename = format!("recording_{}.wav", recording.timestamp);
    let output_path = config.recordings_dir.join(&output_filename);

    // Check if output already exists
    if output_path.exists() {
        return Err(anyhow::anyhow!("Output file already exists: {}", output_filename).into());
    }

    // Determine audio detection status (we can't know for sure, so assume true if file exists)
    let loopback_has_audio = recording.loopback_file.is_some();
    let mic_has_audio = recording.mic_file.is_some();

    // Get file paths or create dummy paths
    let loopback_path = recording
        .loopback_file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No loopback file found for recovery"))?;

    // Create a dummy mic file path if it doesn't exist (merge function handles this)
    let mic_path = recording
        .mic_file
        .clone()
        .unwrap_or_else(|| config.recordings_dir.join(format!("{}_mic.wav", recording.session_id)));

    output.prefixed("Merging", "Merging audio channels...");

    // Merge audio streams
    merge_audio_streams_smart(
        &loopback_path,
        &mic_path,
        &output_path,
        loopback_has_audio,
        mic_has_audio,
        &quality,
    )
    .await
    .context("Failed to merge audio streams")?;

    output.success("Successfully merged audio channels!");

    // Convert to M4A if requested
    let mut final_path = output_path.clone();
    if matches!(format, AudioFormat::M4a) {
        output.prefixed("Converting", "WAV to M4A format...");
        let m4a_path = output_path.with_extension("m4a");

        convert_wav_to_m4a(&output_path, &m4a_path)
            .await
            .context("Failed to convert to M4A")?;

        output.success("Successfully converted to M4A format!");

        // Delete temporary WAV file
        if let Err(e) = std::fs::remove_file(&output_path) {
            tracing::warn!("Failed to delete temporary WAV file: {}", e);
        }

        final_path = m4a_path;
    }

    // Cleanup temporary files
    if let Some(ref loopback) = recording.loopback_file {
        let _ = std::fs::remove_file(loopback);
    }
    if let Some(ref mic) = recording.mic_file {
        let _ = std::fs::remove_file(mic);
    }

    output.info("Temporary files cleaned up");

    Ok(final_path)
}
