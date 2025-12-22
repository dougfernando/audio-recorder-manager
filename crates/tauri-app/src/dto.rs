//! Data Transfer Objects and Response types for Tauri commands

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResponse {
    pub status: String,
    pub session_id: Option<String>,
    pub file_path: Option<String>,
    pub filename: Option<String>,
    pub duration: Option<i64>,
    pub quality: Option<String>,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverResponse {
    pub status: String,
    pub message: String,
    pub recovered: Vec<RecoveredSession>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveredSession {
    pub session_id: String,
    pub output_file: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub devices: Vec<DeviceInfo>,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatusInfo {
    pub status: String,
    pub session_id: Option<String>,
    pub filename: Option<String>,
    pub duration: Option<u64>,
    pub elapsed: Option<u64>,
    pub progress: Option<u64>,
    pub quality: Option<String>,
    pub device: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub frames_captured: Option<u64>,
    pub has_audio: Option<bool>,
    // Per-channel data for dual-channel recording
    pub loopback_frames: Option<u64>,
    pub loopback_has_audio: Option<bool>,
    pub mic_frames: Option<u64>,
    pub mic_has_audio: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub created: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevels {
    pub loopback: f32,
    pub microphone: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    pub points: Vec<f32>,
    pub duration_ms: u64,
    pub channels: u16,
}
