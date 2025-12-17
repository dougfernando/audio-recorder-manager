// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod splash_screen;

use audio_recorder_manager_core::{
    audio_monitor::windows_monitor::AudioLevelMonitor,
    commands::{cancel, record, recover, status, stop},
    config::RecorderConfig,
    domain::{AudioFormat, RecordingDuration},
    logging,
    recorder::RecordingQuality,
    transcription::{load_config, save_config, transcribe_audio, TranscriptionConfig},
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// State to track active recording sessions and audio monitor
struct AppState {
    active_sessions: Mutex<Vec<String>>,
    audio_monitor: Mutex<Option<AudioLevelMonitor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordingResponse {
    status: String,
    session_id: Option<String>,
    file_path: Option<String>,
    filename: Option<String>,
    duration: Option<i64>,
    quality: Option<String>,
    message: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecoverResponse {
    status: String,
    message: String,
    recovered: Vec<RecoveredSession>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecoveredSession {
    session_id: String,
    output_file: String,
    output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceInfo {
    name: String,
    sample_rate: u32,
    channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatusResponse {
    status: String,
    devices: Vec<DeviceInfo>,
    message: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordingStatusInfo {
    status: String,
    session_id: Option<String>,
    filename: Option<String>,
    duration: Option<u64>,
    elapsed: Option<u64>,
    progress: Option<u64>,
    quality: Option<String>,
    device: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    frames_captured: Option<u64>,
    has_audio: Option<bool>,
    // Per-channel data for dual-channel recording
    loopback_frames: Option<u64>,
    loopback_has_audio: Option<bool>,
    mic_frames: Option<u64>,
    mic_has_audio: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordingFile {
    filename: String,
    path: String,
    size: u64,
    created: String,
    format: String,
}

/// Start a new recording
#[tauri::command]
async fn start_recording(
    duration_secs: i64,
    format: String,
    quality: String,
    state: State<'_, AppState>,
) -> Result<RecordingResponse, String> {
    let config = RecorderConfig::new();

    // Parse duration
    let duration = RecordingDuration::from_secs(duration_secs, config.max_manual_duration_secs)
        .map_err(|e| e.to_string())?;

    // Parse format
    let audio_format = AudioFormat::from_str(&format).map_err(|e| e.to_string())?;

    // Parse quality
    let recording_quality = match quality.to_lowercase().as_str() {
        "quick" => RecordingQuality::quick(),
        "standard" => RecordingQuality::standard(),
        "professional" => RecordingQuality::professional(),
        "high" => RecordingQuality::high(),
        _ => {
            return Err(format!(
                "Invalid quality '{}'. Options: quick, standard, professional, high",
                quality
            ))
        }
    };

    // Start recording in background task
    let session_id = chrono::Local::now().format("rec-%Y%m%d_%H%M%S").to_string();
    let session_id_clone = session_id.clone();
    let session_id_clone2 = session_id.clone();

    // Get file path before moving config
    let file_path = config
        .recordings_dir
        .join(format!("recording_{}.{}", session_id, format))
        .to_string_lossy()
        .to_string();
    let filename = format!("recording_{}.{}", session_id_clone, format);

    // Add to active sessions
    {
        match state.active_sessions.lock() {
            Ok(mut sessions) => sessions.push(session_id.clone()),
            Err(e) => {
                tracing::error!(error = %e, "Failed to lock active_sessions mutex (poisoned)");
                return Err(format!(
                    "Internal error: Failed to track recording session: {}",
                    e
                ));
            }
        }
    }

    // Spawn recording task
    tokio::spawn(async move {
        let _result = record::execute(duration, audio_format, recording_quality, config).await;
        if let Err(e) = _result {
            eprintln!("Recording error: {}", e);
        }
    });

    Ok(RecordingResponse {
        status: "success".to_string(),
        session_id: Some(session_id_clone2),
        file_path: Some(file_path),
        filename: Some(filename),
        duration: Some(duration_secs),
        quality: Some(quality),
        message: "Recording started successfully".to_string(),
        error: None,
    })
}

/// Stop an active recording
#[tauri::command]
async fn stop_recording(
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<RecordingResponse, String> {
    let config = RecorderConfig::new();

    // Execute stop command
    stop::execute(session_id.clone(), config)
        .await
        .map_err(|e| e.to_string())?;

    // Remove from active sessions
    match state.active_sessions.lock() {
        Ok(mut sessions) => {
            if let Some(sid) = &session_id {
                sessions.retain(|s| s != sid);
            } else {
                sessions.clear();
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to lock active_sessions mutex in stop_recording");
            // Continue anyway since stop command already executed
        }
    }

    Ok(RecordingResponse {
        status: "success".to_string(),
        session_id,
        file_path: None,
        filename: None,
        duration: None,
        quality: None,
        message: "Recording stopped successfully".to_string(),
        error: None,
    })
}

/// Cancel an active recording without processing
#[tauri::command]
async fn cancel_recording(
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<RecordingResponse, String> {
    let config = RecorderConfig::new();

    // Execute cancel command
    cancel::execute(session_id.clone(), config)
        .await
        .map_err(|e| e.to_string())?;

    // Remove from active sessions
    match state.active_sessions.lock() {
        Ok(mut sessions) => {
            if let Some(sid) = &session_id {
                sessions.retain(|s| s != sid);
            } else {
                sessions.clear();
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to lock active_sessions mutex in cancel_recording");
            // Continue anyway since cancel command already executed
        }
    }

    Ok(RecordingResponse {
        status: "success".to_string(),
        session_id,
        file_path: None,
        filename: None,
        duration: None,
        quality: None,
        message: "Recording cancelled successfully".to_string(),
        error: None,
    })
}

/// Get device status
#[tauri::command]
async fn get_status() -> Result<StatusResponse, String> {
    // Execute status command (this prints to stdout in the original)
    // We'll need to capture the output or restructure it
    status::execute().await.map_err(|e| e.to_string())?;

    // For now, return a basic response
    // TODO: Modify status::execute to return data instead of printing
    Ok(StatusResponse {
        status: "success".to_string(),
        devices: vec![],
        message: "Status retrieved successfully".to_string(),
        error: None,
    })
}

/// Recover interrupted recordings
#[tauri::command]
async fn recover_recordings(
    session_id: Option<String>,
    format: Option<String>,
) -> Result<RecoverResponse, String> {
    let config = RecorderConfig::new();

    // Parse format if provided
    let target_format = if let Some(fmt) = format {
        Some(AudioFormat::from_str(&fmt).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Execute recover command
    recover::execute(session_id, target_format, config)
        .await
        .map_err(|e| e.to_string())?;

    // TODO: Capture actual recovery results
    Ok(RecoverResponse {
        status: "success".to_string(),
        message: "Recovery completed successfully".to_string(),
        recovered: vec![],
        error: None,
    })
}

/// Get recording status from status file
#[tauri::command]
async fn get_recording_status(session_id: String) -> Result<RecordingStatusInfo, String> {
    let config = RecorderConfig::new();
    let status_file = config.status_dir.join(format!("{}.json", session_id));

    if !status_file.exists() {
        return Ok(RecordingStatusInfo {
            status: "not_found".to_string(),
            session_id: Some(session_id),
            filename: None,
            duration: None,
            elapsed: None,
            progress: None,
            quality: None,
            device: None,
            sample_rate: None,
            channels: None,
            frames_captured: None,
            has_audio: None,
            loopback_frames: None,
            loopback_has_audio: None,
            mic_frames: None,
            mic_has_audio: None,
        });
    }

    // Read status file
    let content = std::fs::read_to_string(status_file).map_err(|e| e.to_string())?;
    let status_data: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(RecordingStatusInfo {
        status: status_data["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        session_id: status_data["session_id"].as_str().map(|s| s.to_string()),
        filename: status_data["filename"].as_str().map(|s| s.to_string()),
        duration: status_data["duration"].as_u64(),
        elapsed: status_data["elapsed"].as_u64(),
        progress: status_data["progress"].as_u64(),
        quality: status_data["quality"].as_str().map(|s| s.to_string()),
        device: status_data["device"].as_str().map(|s| s.to_string()),
        sample_rate: status_data["sample_rate"].as_u64().map(|v| v as u32),
        channels: status_data["channels"].as_u64().map(|v| v as u16),
        frames_captured: status_data["frames_captured"].as_u64(),
        has_audio: status_data["has_audio"].as_bool(),
        loopback_frames: status_data["loopback_frames"].as_u64(),
        loopback_has_audio: status_data["loopback_has_audio"].as_bool(),
        mic_frames: status_data["mic_frames"].as_u64(),
        mic_has_audio: status_data["mic_has_audio"].as_bool(),
    })
}

/// List all recordings
#[tauri::command]
async fn list_recordings() -> Result<Vec<RecordingFile>, String> {
    let config = RecorderConfig::new();
    let recordings_dir = config.recordings_dir;

    if !recordings_dir.exists() {
        return Ok(vec![]);
    }

    let mut recordings = vec![];

    let entries = std::fs::read_dir(recordings_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        // Skip temporary files and status files
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.contains("_loopback")
                || filename_str.contains("_mic")
                || filename_str.ends_with(".json")
            {
                continue;
            }
        }

        // Only include audio files
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "wav" || ext_str == "m4a" {
                let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
                let created = metadata
                    .created()
                    .ok()
                    .and_then(|time| {
                        chrono::DateTime::<chrono::Local>::from(time)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                            .into()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                if let Some(filename) = path.file_name() {
                    recordings.push(RecordingFile {
                        filename: filename.to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                        created,
                        format: ext_str,
                    });
                } else {
                    tracing::warn!(path = ?path, "Skipping file with invalid filename");
                }
            }
        }
    }

    // Sort by created date (newest first)
    recordings.sort_by(|a, b| b.created.cmp(&a.created));

    tracing::info!("Loaded {} recordings", recordings.len());

    Ok(recordings)
}

/// Get a specific recording by path
#[tauri::command]
async fn get_recording(file_path: String) -> Result<RecordingFile, String> {
    use std::path::Path;

    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("Recording not found: {}", file_path));
    }

    // Get file metadata
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;

    // Get file extension
    let ext = path
        .extension()
        .ok_or_else(|| "Invalid file format".to_string())?
        .to_string_lossy()
        .to_lowercase();

    // Verify it's an audio file
    if ext != "wav" && ext != "m4a" {
        return Err("Invalid audio file format".to_string());
    }

    // Get creation date
    let created = metadata
        .created()
        .ok()
        .and_then(|time| {
            chrono::DateTime::<chrono::Local>::from(time)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .into()
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let filename = path
        .file_name()
        .ok_or_else(|| "Invalid file path: no filename".to_string())?
        .to_string_lossy()
        .to_string();

    Ok(RecordingFile {
        filename,
        path: path.to_string_lossy().to_string(),
        size: metadata.len(),
        created,
        format: ext,
    })
}

/// Get list of active recording sessions
#[tauri::command]
async fn get_active_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    match state.active_sessions.lock() {
        Ok(sessions) => Ok(sessions.clone()),
        Err(e) => {
            tracing::error!(error = %e, "Failed to lock active_sessions mutex in get_active_sessions");
            Err(format!(
                "Internal error: Failed to get active sessions: {}",
                e
            ))
        }
    }
}

/// Open a recording file with the default application
#[tauri::command]
async fn open_recording(file_path: String) -> Result<String, String> {
    use std::path::Path;

    let path = Path::new(&file_path);

    // Verify the file exists
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Open the file with the default application
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(&["/C", "start", "", &file_path]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd.spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("Opening files is only supported on Windows".to_string());
    }

    Ok(format!("Opened: {}", file_path))
}

/// Open the folder containing a recording file
#[tauri::command]
async fn open_folder(file_path: String) -> Result<String, String> {
    use std::path::Path;

    let path = Path::new(&file_path);

    // Get the parent directory
    let folder = if path.is_dir() {
        path
    } else {
        path.parent().ok_or_else(|| "Failed to get parent directory".to_string())?
    };

    // Open the folder
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(folder);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd.spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("Opening folders is only supported on Windows".to_string());
    }

    Ok(format!("Opened folder: {}", folder.display()))
}

/// Delete a recording file
#[tauri::command]
async fn delete_recording(file_path: String) -> Result<String, String> {
    use std::path::Path;

    let path = Path::new(&file_path);

    // Verify the file exists
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Delete the audio file
    std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;

    // Also delete the transcript if it exists (in transcriptions directory)
    let config = RecorderConfig::new();
    if let Some(file_stem) = path.file_stem() {
        let transcript_path = config
            .transcriptions_dir
            .join(file_stem)
            .with_extension("md");
        if transcript_path.exists() {
            tracing::info!("Deleting associated transcript: {:?}", transcript_path);
            std::fs::remove_file(&transcript_path)
                .map_err(|e| tracing::warn!("Failed to delete transcript: {}", e))
                .ok();
        }
    }

    Ok(format!("Successfully deleted: {}", file_path))
}

/// Rename a recording file
#[tauri::command]
async fn rename_recording(old_path: String, new_filename: String) -> Result<RecordingFile, String> {
    use std::path::Path;

    let old_path = Path::new(&old_path);

    // 1. Validate new filename
    if new_filename.is_empty() || new_filename.contains('/') || new_filename.contains('\\') {
        return Err("Invalid new filename".to_string());
    }

    // 2. Check if old file exists
    if !old_path.exists() {
        return Err(format!("File not found: {}", old_path.display()));
    }

    // 3. Construct new path
    let parent_dir = old_path
        .parent()
        .ok_or("Could not determine parent directory")?;
    let old_extension = old_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let new_path = parent_dir.join(&new_filename).with_extension(old_extension);

    // 4. Check if new path already exists
    if new_path.exists() {
        return Err(format!(
            "A file named '{}' already exists.",
            new_path.display()
        ));
    }

    // 5. Rename the audio file
    std::fs::rename(old_path, &new_path)
        .map_err(|e| format!("Failed to rename audio file: {}", e))?;

    // 6. Rename the transcript file
    let config = RecorderConfig::new();
    let old_file_stem = old_path.file_stem().ok_or("Could not get file stem")?;
    let new_file_stem = Path::new(&new_filename)
        .file_stem()
        .ok_or("Could not get file stem from new filename")?;
    let old_transcript_path = config
        .transcriptions_dir
        .join(old_file_stem)
        .with_extension("md");

    if old_transcript_path.exists() {
        let new_transcript_path = config
            .transcriptions_dir
            .join(new_file_stem)
            .with_extension("md");
        std::fs::rename(&old_transcript_path, &new_transcript_path).map_err(|e| {
            // Try to roll back the audio file rename on transcript rename failure
            if let Err(rollback_err) = std::fs::rename(&new_path, old_path) {
                tracing::error!("Failed to rollback audio file rename: {}", rollback_err);
            }
            format!("Failed to rename transcript file: {}", e)
        })?;
    }

    // 7. Return the new RecordingFile object
    get_recording(new_path.to_string_lossy().to_string()).await
}

/// Load transcription configuration
#[tauri::command]
async fn load_transcription_config() -> Result<TranscriptionConfig, String> {
    load_config().map_err(|e| e.to_string())
}

/// Save transcription configuration
#[tauri::command]
async fn save_transcription_config(config: TranscriptionConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

/// Transcribe an audio file using Gemini API
#[tauri::command]
async fn transcribe_recording(
    file_path: String,
    session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    use std::path::Path;

    tracing::info!("Transcription requested for: {}", file_path);

    // Load configuration
    let config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    // Validate API key
    if config.api_key.is_empty() {
        tracing::error!("Transcription failed: API key not configured");
        return Err("API key not configured. Please configure it in Settings.".to_string());
    }

    // Validate file path
    let path = Path::new(&file_path);
    if !path.exists() {
        tracing::error!("Transcription failed: File not found at {}", file_path);
        return Err(format!("File not found: {}", file_path));
    }

    // Generate session ID if not provided
    let session_id = session_id.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to get system time, using fallback");
                std::time::Duration::from_secs(0)
            })
            .as_secs();
        format!("transcribe_{}", timestamp)
    });

    tracing::info!("Starting transcription with session ID: {}", session_id);

    // Load recorder config to get transcriptions directory
    let recorder_config = RecorderConfig::new();

    // Run transcription (now async, no need for spawn_blocking)
    let path_clone = path.to_path_buf();
    let transcriptions_dir = recorder_config.transcriptions_dir.clone();
    let api_key = config.api_key.clone();
    let model = config.model.clone();
    let prompt = config.prompt.clone();
    let optimize = config.optimize_audio;
    let session_id_clone = session_id.clone();

    transcribe_audio(
        &path_clone,
        &transcriptions_dir,
        &recorder_config.status_dir,
        &api_key,
        &model,
        &prompt,
        optimize,
        &session_id_clone,
    )
    .await
    .and_then(|result| {
        tracing::info!("Transcription completed successfully");
        serde_json::to_value(result).map_err(|e| {
            tracing::error!(error = %e, "Failed to serialize transcription result");
            anyhow::anyhow!("Failed to serialize transcription result: {}", e)
        })
    })
    .map_err(|e| {
        tracing::error!(error = %e, "Transcription failed");
        e.to_string()
    })
}

/// Read transcript file content
#[tauri::command]
async fn read_transcript(file_path: String) -> Result<String, String> {
    use std::path::Path;

    tracing::info!("Reading transcript from: {}", file_path);

    let path = Path::new(&file_path);
    if !path.exists() {
        tracing::error!("Transcript file not found: {}", file_path);
        return Err(format!("Transcript file not found: {}", file_path));
    }

    std::fs::read_to_string(path).map_err(|e| {
        tracing::error!("Failed to read transcript file: {}", e);
        format!("Failed to read transcript: {}", e)
    })
}

/// Check if a transcript exists for a given audio file
#[tauri::command]
async fn check_transcript_exists(file_path: String) -> Result<bool, String> {
    use std::path::Path;

    // Load recorder config to get transcriptions directory
    let config = RecorderConfig::new();

    let path = Path::new(&file_path);
    let file_stem = path
        .file_stem()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let transcript_path = config
        .transcriptions_dir
        .join(file_stem)
        .with_extension("md");

    Ok(transcript_path.exists())
}

/// Get the transcript file path for a given audio file
#[tauri::command]
async fn get_transcript_path(file_path: String) -> Result<String, String> {
    use std::path::Path;

    // Load recorder config to get transcriptions directory
    let config = RecorderConfig::new();

    let path = Path::new(&file_path);
    let file_stem = path
        .file_stem()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let transcript_path = config
        .transcriptions_dir
        .join(file_stem)
        .with_extension("md");

    Ok(transcript_path.to_string_lossy().to_string())
}

/// Get transcription status for a session
#[tauri::command]
async fn get_transcription_status(session_id: String) -> Result<Option<serde_json::Value>, String> {
    use audio_recorder_manager_core::transcription::read_transcription_status;

    let recorder_config = RecorderConfig::new();

    read_transcription_status(&recorder_config.status_dir, &session_id)
        .and_then(|status| {
            status.map(|s| {
                serde_json::to_value(s).map_err(|e| {
                    tracing::error!(error = %e, session_id = %session_id, "Failed to serialize transcription status");
                    anyhow::anyhow!("Failed to serialize status: {}", e)
                })
            })
            .transpose()
        })
        .map_err(|e| {
            tracing::error!(error = %e, session_id = %session_id, "Failed to read transcription status");
            e.to_string()
        })
}

/// Get transcription progress for a session (alias for get_transcription_status)
#[tauri::command]
async fn get_transcription_progress(session_id: String) -> Result<Option<serde_json::Value>, String> {
    get_transcription_status(session_id).await
}

/// Load recorder configuration (paths for recordings and transcriptions)
#[tauri::command]
async fn load_recorder_config() -> Result<serde_json::Value, String> {
    let config = RecorderConfig::new();
    serde_json::to_value(&config).map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Save recorder configuration (storage path)
#[tauri::command]
async fn save_recorder_config(storage_dir: String) -> Result<(), String> {
    let config = RecorderConfig::from_storage_dir(std::path::PathBuf::from(storage_dir));

    config
        .ensure_directories()
        .map_err(|e| format!("Failed to create directories: {}", e))?;

    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Pick a folder using native dialog
#[tauri::command]
async fn pick_folder(
    app: tauri::AppHandle,
    default_path: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let dialog = app.dialog().file();

    let dialog = if let Some(path) = default_path {
        dialog.set_directory(path)
    } else {
        dialog
    };

    Ok(dialog.blocking_pick_folder().map(|p| p.to_string()))
}

/// Start monitoring audio input levels
#[tauri::command]
async fn start_audio_monitor(state: State<'_, AppState>) -> Result<(), String> {
    // Check if we need to clean up existing monitor
    let needs_cleanup = {
        let monitor = state
            .audio_monitor
            .lock()
            .map_err(|e| format!("Failed to lock audio_monitor mutex: {}", e))?;
        monitor.is_some()
    };

    // Stop existing monitor if any
    if needs_cleanup {
        {
            let mut monitor = state
                .audio_monitor
                .lock()
                .map_err(|e| format!("Failed to lock audio_monitor mutex: {}", e))?;
            *monitor = None;
        }
        // Give it a moment to clean up
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Start new monitor
    let new_monitor =
        AudioLevelMonitor::new().map_err(|e| format!("Failed to start audio monitor: {}", e))?;

    {
        let mut monitor = state
            .audio_monitor
            .lock()
            .map_err(|e| format!("Failed to lock audio_monitor mutex: {}", e))?;
        *monitor = Some(new_monitor);
    }
    tracing::info!("Audio level monitor started");

    Ok(())
}

/// Stop monitoring audio input levels
#[tauri::command]
async fn stop_audio_monitor(state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state
        .audio_monitor
        .lock()
        .map_err(|e| format!("Failed to lock audio_monitor mutex: {}", e))?;

    if monitor.is_some() {
        *monitor = None;
        tracing::info!("Audio level monitor stopped");
    }

    Ok(())
}

/// Get current audio input levels
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AudioLevels {
    loopback: f32,
    microphone: f32,
}

#[tauri::command]
async fn get_audio_levels(state: State<'_, AppState>) -> Result<AudioLevels, String> {
    let monitor = state
        .audio_monitor
        .lock()
        .map_err(|e| format!("Failed to lock audio_monitor mutex: {}", e))?;

    if let Some(ref m) = *monitor {
        Ok(AudioLevels {
            loopback: m.get_loopback_level(),
            microphone: m.get_microphone_level(),
        })
    } else {
        Ok(AudioLevels {
            loopback: 0.0,
            microphone: 0.0,
        })
    }
}

/// Quit the application
#[tauri::command]
async fn quit_app(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Stop audio monitor if running
    if let Ok(mut monitor) = state.audio_monitor.lock() {
        *monitor = None;
        tracing::info!("Audio monitor stopped during app quit");
    }

    // Destroy the window properly instead of just closing
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.destroy();
    }

    // Give resources a moment to clean up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("Application quitting via quit command");
    app.exit(0);

    Ok(())
}

/// Generate waveform data from an audio file using ffmpeg
#[tauri::command]
async fn generate_waveform(file_path: String, samples: Option<usize>) -> Result<Vec<f32>, String> {
    use std::path::Path;
    use tokio::process::Command;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Default to 100 samples for the waveform
    let num_samples = samples.unwrap_or(100);

    tracing::info!(
        "Generating waveform for: {} with {} samples",
        file_path,
        num_samples
    );

    // Use ffmpeg to extract audio data and calculate peak values
    // We'll use ffmpeg to decode audio and output raw PCM data, then calculate peaks
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(&file_path)
        .arg("-ac")
        .arg("1") // Convert to mono
        .arg("-f")
        .arg("f32le") // Output as 32-bit float PCM
        .arg("-ar")
        .arg("8000") // Downsample to 8kHz for faster processing
        .arg("-");

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let output = cmd.output().await.map_err(|e| {
        tracing::error!("Failed to run ffmpeg: {}", e);
        format!(
            "Failed to run ffmpeg: {}. Make sure ffmpeg is installed and in PATH.",
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ffmpeg failed: {}", stderr);
        return Err(format!("ffmpeg failed: {}", stderr));
    }

    // Parse the raw PCM data (f32 little-endian)
    let raw_data = output.stdout;
    let sample_count = raw_data.len() / 4; // 4 bytes per f32

    if sample_count == 0 {
        return Err("No audio data found in file".to_string());
    }

    // Convert bytes to f32 samples
    let mut audio_samples: Vec<f32> = Vec::with_capacity(sample_count);
    for chunk in raw_data.chunks_exact(4) {
        let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        audio_samples.push(sample.abs()); // Use absolute value for waveform
    }

    // Calculate peak values for visualization
    let samples_per_bar = (sample_count as f32 / num_samples as f32).ceil() as usize;
    let mut waveform: Vec<f32> = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let start = i * samples_per_bar;
        let end = ((i + 1) * samples_per_bar).min(sample_count);

        if start >= sample_count {
            waveform.push(0.0);
            continue;
        }

        // Calculate peak (max) value in this chunk
        let peak = audio_samples[start..end]
            .iter()
            .fold(0.0f32, |max, &sample| max.max(sample));

        waveform.push(peak);
    }

    // Normalize waveform to 0.0-1.0 range
    let max_peak = waveform.iter().fold(0.0f32, |max, &val| max.max(val));
    if max_peak > 0.0 {
        for val in &mut waveform {
            *val /= max_peak;
        }
    }

    tracing::info!(
        "Waveform generated successfully with {} bars",
        waveform.len()
    );

    Ok(waveform)
}

/// Set up system tray with menu
fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let app_handle = app.handle();

    // Create menu items
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide Window", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;

    // Quick recording submenu
    let record_30s = MenuItem::with_id(app, "record_30s", "30 seconds", true, None::<&str>)?;
    let record_1m = MenuItem::with_id(app, "record_1m", "1 minute", true, None::<&str>)?;
    let record_5m = MenuItem::with_id(app, "record_5m", "5 minutes", true, None::<&str>)?;
    let record_10m = MenuItem::with_id(app, "record_10m", "10 minutes", true, None::<&str>)?;

    let quick_record_menu =
        Submenu::with_items(app, "Quick Record", true, &[&record_30s, &record_1m, &record_5m, &record_10m])?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let open_recordings = MenuItem::with_id(app, "open_recordings", "Open Recordings Folder", true, None::<&str>)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(
        app,
        &[
            &show_hide,
            &separator1,
            &quick_record_menu,
            &separator2,
            &open_recordings,
            &separator3,
            &quit,
        ],
    )?;

    // Create tray icon
    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Audio Recorder Manager")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "record_30s" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-start-recording", serde_json::json!({"duration": 30}));
                }
            }
            "record_1m" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-start-recording", serde_json::json!({"duration": 60}));
                }
            }
            "record_5m" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-start-recording", serde_json::json!({"duration": 300}));
                }
            }
            "record_10m" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("tray-start-recording", serde_json::json!({"duration": 600}));
                }
            }
            "open_recordings" => {
                let config = RecorderConfig::new();
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(config.recordings_dir)
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(config.recordings_dir)
                        .spawn();
                }
            }
            "quit" => {
                // Properly cleanup resources before exiting to avoid Chrome window class errors

                // Stop audio monitor if running
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(mut monitor) = state.audio_monitor.lock() {
                        *monitor = None;
                        tracing::info!("Audio monitor stopped during app quit");
                    }
                }

                // Destroy the window properly instead of just closing
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.destroy();
                }

                // Give resources a moment to clean up
                std::thread::sleep(std::time::Duration::from_millis(100));

                tracing::info!("Application quitting via tray menu");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Set up file watcher for status directory
fn setup_status_watcher(app_handle: tauri::AppHandle) {
    let config = RecorderConfig::new();
    let status_dir = config.status_dir.clone();

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(error = %e, "Failed to create file watcher for status directory");
                eprintln!("ERROR: Failed to create file watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&status_dir, RecursiveMode::NonRecursive) {
            tracing::error!(error = %e, status_dir = ?status_dir, "Failed to watch status directory");
            eprintln!("ERROR: Failed to watch status directory: {}", e);
            return;
        }

        for result in rx {
            match result {
                Ok(Event {
                    kind: EventKind::Modify(_) | EventKind::Create(_),
                    paths,
                    ..
                }) => {
                    for path in paths {
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            // Read and emit status update
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(status) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    let _ = app_handle.emit("recording-status-update", status);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

fn main() {
    let app_start = std::time::Instant::now();

    // Optimize WebView2 initialization to reduce startup delay
    // Use a dedicated user data folder in temp to avoid permission/access delays
    let user_data_folder = std::env::temp_dir().join("audio-recorder-webview2");
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", user_data_folder);

    // Initialize dual-output logging (file + terminal in debug builds)
    let enable_terminal = cfg!(debug_assertions);
    if let Err(e) = logging::init_tauri_logging(None, enable_terminal) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    // Set up panic hook to log panics with full backtrace before crashing
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();

        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "Unknown location".to_string()
        };

        // Log to both tracing (file) and stderr (terminal)
        tracing::error!(
            message = %message,
            location = %location,
            backtrace = %backtrace,
            "APPLICATION PANIC - The application has crashed"
        );
        eprintln!("\n!!! APPLICATION PANIC !!!");
        eprintln!("Message: {}", message);
        eprintln!("Location: {}", location);
        eprintln!("\nBacktrace:\n{}", backtrace);
        eprintln!("\nCheck log files at: {:?}", logging::get_log_dir());
    }));

    let log_dir = logging::get_log_dir();
    tracing::info!("========================================");
    tracing::info!("Tauri application starting...");
    tracing::info!(log_dir = ?log_dir, "Logs directory");
    tracing::info!(elapsed = ?app_start.elapsed(), "[TIMING] App start");

    // Create native splash screen (shows instantly, no WebView2 dependency)
    #[cfg(windows)]
    let splash = {
        tracing::info!(
            "[TIMING] Creating native splash screen: {:?}",
            app_start.elapsed()
        );
        match splash_screen::SplashScreen::new() {
            Ok(s) => {
                tracing::info!(
                    "[TIMING] Native splash screen created and visible: {:?}",
                    app_start.elapsed()
                );
                Some(s)
            }
            Err(e) => {
                tracing::warn!("Failed to create splash screen: {}", e);
                None
            }
        }
    };
    #[cfg(not(windows))]
    let splash: Option<()> = None;

    tracing::info!("[TIMING] Creating Tauri builder: {:?}", app_start.elapsed());
    let builder = tauri::Builder::default();
    let builder = builder.plugin(tauri_plugin_shell::init());
    let builder = builder.plugin(tauri_plugin_dialog::init());
    let builder = builder.manage(AppState {
        active_sessions: Mutex::new(Vec::new()),
        audio_monitor: Mutex::new(None),
    });

    tracing::info!(
        "[TIMING] Configuring setup handler: {:?}",
        app_start.elapsed()
    );
    let builder = builder.setup({
        let splash_opt = splash;
        move |app| {

            // Ensure storage directories exist
            let config = RecorderConfig::new();

            if let Err(e) = config.ensure_directories() {
                tracing::error!(error = %e, "Failed to create storage directories - application may not function correctly");
                eprintln!("ERROR: Failed to create storage directories: {}", e);
                eprintln!("Check log files at: {:?}", logging::get_log_dir());
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            // Set up status file watcher
            setup_status_watcher(app.handle().clone());

            // Set up system tray
            setup_system_tray(app)?;

            // Handle window close event to minimize to tray instead of exiting
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent default close behavior
                        api.prevent_close();
                        // Hide window instead
                        let _ = window_clone.hide();
                    }
                });
            }

            // Close splash screen now that main window is ready
            #[cfg(windows)]
            if let Some(s) = splash_opt {
                s.close();
            }

            Ok(())
        }
    });

    let builder = builder.invoke_handler(tauri::generate_handler![
        start_recording,
        stop_recording,
        cancel_recording,
        get_status,
        recover_recordings,
        get_recording_status,
        list_recordings,
        get_recording,
        get_active_sessions,
        open_recording,
        open_folder,
        delete_recording,
        rename_recording,
        load_transcription_config,
        save_transcription_config,
        transcribe_recording,
        read_transcript,
        check_transcript_exists,
        get_transcript_path,
        get_transcription_status,
        get_transcription_progress,
        load_recorder_config,
        save_recorder_config,
        pick_folder,
        start_audio_monitor,
        stop_audio_monitor,
        get_audio_levels,
        generate_waveform,
        quit_app,
    ]);

    tracing::info!(
        "[TIMING] Starting Tauri application run loop: {:?}",
        app_start.elapsed()
    );
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    tracing::info!("[TIMING] Application exited: {:?}", app_start.elapsed());
}
