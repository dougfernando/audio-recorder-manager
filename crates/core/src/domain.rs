use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{RecorderError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    M4a,
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioFormat::Wav => write!(f, "wav"),
            AudioFormat::M4a => write!(f, "m4a"),
        }
    }
}

impl FromStr for AudioFormat {
    type Err = RecorderError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "wav" => Ok(AudioFormat::Wav),
            "m4a" => Ok(AudioFormat::M4a),
            _ => Err(RecorderError::InvalidParameter(format!(
                "Unsupported audio format '{}'. Supported formats: wav, m4a",
                s
            ))),
        }
    }
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::M4a => "m4a",
        }
    }

    pub fn codec(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "pcm",
            AudioFormat::M4a => "aac",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingDuration {
    Fixed(u64),        // seconds
    Manual { max: u64 }, // -1 becomes this
}

impl RecordingDuration {
    pub fn from_secs(duration: i64, max_manual_duration: u64) -> Result<Self> {
        match duration {
            -1 => Ok(RecordingDuration::Manual { max: max_manual_duration }),
            d if d > 0 => Ok(RecordingDuration::Fixed(d as u64)),
            d => Err(RecorderError::InvalidParameter(format!(
                "Duration must be -1 (manual mode) or a positive number, got: {}",
                d
            ))),
        }
    }

    pub fn effective_duration(&self) -> u64 {
        match self {
            RecordingDuration::Fixed(d) => *d,
            RecordingDuration::Manual { max } => *max,
        }
    }

    pub fn to_api_value(&self) -> i64 {
        match self {
            RecordingDuration::Fixed(d) => *d as i64,
            RecordingDuration::Manual { .. } => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        Self(format!("rec-{}", timestamp))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct RecordingSession {
    pub id: SessionId,
    pub format: AudioFormat,
    pub quality: crate::recorder::RecordingQuality,
    pub duration: RecordingDuration,
    pub started_at: DateTime<Local>,
}

impl RecordingSession {
    pub fn new(
        format: AudioFormat,
        quality: crate::recorder::RecordingQuality,
        duration: RecordingDuration,
    ) -> Self {
        Self {
            id: SessionId::new(),
            format,
            quality,
            duration,
            started_at: Local::now(),
        }
    }

    pub fn filename(&self) -> String {
        let timestamp = self.started_at.format("%Y%m%d_%H%M%S");
        format!("recording_{}.{}", timestamp, self.format.extension())
    }

    pub fn temp_filename(&self) -> String {
        let timestamp = self.started_at.format("%Y%m%d_%H%M%S");
        format!("recording_{}.wav", timestamp)
    }
}
