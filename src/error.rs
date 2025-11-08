use std::io;

#[derive(Debug, thiserror::Error)]
pub enum RecorderError {
    #[error("Audio device error: {0}")]
    DeviceError(String),

    #[error("Recording error: {0}")]
    RecordingError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RecorderError>;
