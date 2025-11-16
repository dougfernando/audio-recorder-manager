// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use audio_recorder_manager_core::{
    commands::{record, recover, status, stop},
    config::RecorderConfig,
    domain::{AudioFormat, RecordingDuration},
    recorder::RecordingQuality,
    transcription::{load_config, save_config, transcribe_audio, TranscriptionConfig},
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{Emitter, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

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
    std::fs::remove_file(path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    // Also delete the transcript if it exists (in transcriptions directory)
    let config = RecorderConfig::new();
    if let Some(file_stem) = path.file_stem() {
        let transcript_path = config.transcriptions_dir.join(file_stem).with_extension("md");
        if transcript_path.exists() {
            log::info!("Deleting associated transcript: {:?}", transcript_path);
            std::fs::remove_file(&transcript_path)
                .map_err(|e| log::warn!("Failed to delete transcript: {}", e))
                .ok();
        }
    }

    Ok(format!("Successfully deleted: {}", file_path))
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

    log::info!("Transcription requested for: {}", file_path);

    // Load configuration
    let config = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    // Validate API key
    if config.api_key.is_empty() {
        log::error!("Transcription failed: API key not configured");
        return Err("API key not configured. Please configure it in Settings.".to_string());
    }

    // Validate file path
    let path = Path::new(&file_path);
    if !path.exists() {
        log::error!("Transcription failed: File not found at {}", file_path);
        return Err(format!("File not found: {}", file_path));
    }

    // Generate session ID if not provided
    let session_id = session_id.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("transcribe_{}", timestamp)
    });

    log::info!("Starting transcription with session ID: {}", session_id);

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

    transcribe_audio(&path_clone, &transcriptions_dir, &api_key, &model, &prompt, optimize, &session_id_clone)
        .await
        .map(|result| {
            log::info!("Transcription completed successfully");
            serde_json::to_value(result).unwrap()
        })
        .map_err(|e| {
            log::error!("Transcription failed: {}", e);
            e.to_string()
        })
}

/// Read transcript file content
#[tauri::command]
async fn read_transcript(file_path: String) -> Result<String, String> {
    use std::path::Path;

    log::info!("Reading transcript from: {}", file_path);

    let path = Path::new(&file_path);
    if !path.exists() {
        log::error!("Transcript file not found: {}", file_path);
        return Err(format!("Transcript file not found: {}", file_path));
    }

    std::fs::read_to_string(path)
        .map_err(|e| {
            log::error!("Failed to read transcript file: {}", e);
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
    let file_stem = path.file_stem()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let transcript_path = config.transcriptions_dir.join(file_stem).with_extension("md");

    Ok(transcript_path.exists())
}

/// Get the transcript file path for a given audio file
#[tauri::command]
async fn get_transcript_path(file_path: String) -> Result<String, String> {
    use std::path::Path;

    // Load recorder config to get transcriptions directory
    let config = RecorderConfig::new();

    let path = Path::new(&file_path);
    let file_stem = path.file_stem()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let transcript_path = config.transcriptions_dir.join(file_stem).with_extension("md");

    Ok(transcript_path.to_string_lossy().to_string())
}

/// Get transcription status for a session
#[tauri::command]
async fn get_transcription_status(session_id: String) -> Result<Option<serde_json::Value>, String> {
    use audio_recorder_manager_core::transcription::read_transcription_status;

    read_transcription_status(&session_id)
        .map(|status| status.map(|s| serde_json::to_value(s).unwrap()))
        .map_err(|e| e.to_string())
}

/// Load recorder configuration (paths for recordings and transcriptions)
#[tauri::command]
async fn load_recorder_config() -> Result<serde_json::Value, String> {
    let config = RecorderConfig::new();
    serde_json::to_value(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Save recorder configuration (paths for recordings and transcriptions)
#[tauri::command]
async fn save_recorder_config(
    recordings_dir: String,
    transcriptions_dir: String,
) -> Result<(), String> {
    let mut config = RecorderConfig::new();
    config.recordings_dir = std::path::PathBuf::from(recordings_dir);
    config.transcriptions_dir = std::path::PathBuf::from(transcriptions_dir);

    config.ensure_directories()
        .map_err(|e| format!("Failed to create directories: {}", e))?;

    config.save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Pick a folder using native dialog
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle, default_path: Option<String>) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let dialog = app.dialog().file();

    let dialog = if let Some(path) = default_path {
        dialog.set_directory(path)
    } else {
        dialog
    };

    Ok(dialog.blocking_pick_folder().map(|p| p.to_string()))
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
    // Initialize logger to write to file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("tauri_app.log")
        .expect("Failed to open log file");

    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Tauri application starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            active_sessions: Mutex::new(Vec::new()),
        })
        .setup(|app| {
            // Ensure storage directories exist
            let config = RecorderConfig::new();
            config.ensure_directories().expect("Failed to create storage directories");

            // Set up status file watcher
            setup_status_watcher(app.handle().clone());

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
            open_recording,
            delete_recording,
            load_transcription_config,
            save_transcription_config,
            transcribe_recording,
            read_transcript,
            check_transcript_exists,
            get_transcript_path,
            get_transcription_status,
            load_recorder_config,
            save_recorder_config,
            pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
