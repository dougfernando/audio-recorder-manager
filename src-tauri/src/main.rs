// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use audio_recorder_manager::{
    commands::{record, recover, status, stop},
    config::RecorderConfig,
    domain::{AudioFormat, RecordingDuration},
    recorder::RecordingQuality,
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{Manager, State};

// State to track active recording sessions
struct AppState {
    active_sessions: Mutex<Vec<String>>,
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
        let mut sessions = state.active_sessions.lock().unwrap();
        sessions.push(session_id.clone());
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
    if let Some(sid) = &session_id {
        let mut sessions = state.active_sessions.lock().unwrap();
        sessions.retain(|s| s != sid);
    } else {
        let mut sessions = state.active_sessions.lock().unwrap();
        sessions.clear();
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
        status: status_data["status"].as_str().unwrap_or("unknown").to_string(),
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
            if filename_str.contains("_loopback") || filename_str.contains("_mic") || filename_str.ends_with(".json") {
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

                recordings.push(RecordingFile {
                    filename: path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    created,
                    format: ext_str,
                });
            }
        }
    }

    // Sort by created date (newest first)
    recordings.sort_by(|a, b| b.created.cmp(&a.created));

    Ok(recordings)
}

/// Get list of active recording sessions
#[tauri::command]
async fn get_active_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let sessions = state.active_sessions.lock().unwrap();
    Ok(sessions.clone())
}

/// Set up file watcher for status directory
fn setup_status_watcher(app_handle: tauri::AppHandle) {
    let config = RecorderConfig::new();
    let status_dir = config.status_dir.clone();

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .expect("Failed to create file watcher");

        watcher
            .watch(&status_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch status directory");

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
                                if let Ok(status) = serde_json::from_str::<serde_json::Value>(&content) {
                                    app_handle
                                        .emit_all("recording-status-update", status)
                                        .ok();
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
    // Initialize logger
    env_logger::init();

    tauri::Builder::default()
        .manage(AppState {
            active_sessions: Mutex::new(Vec::new()),
        })
        .setup(|app| {
            // Ensure storage directories exist
            let config = RecorderConfig::new();
            config.ensure_directories().expect("Failed to create storage directories");

            // Set up status file watcher
            setup_status_watcher(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            get_status,
            recover_recordings,
            get_recording_status,
            list_recordings,
            get_active_sessions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
