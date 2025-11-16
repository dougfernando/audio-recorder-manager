use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecorderConfig {
    pub recordings_dir: PathBuf,
    pub transcriptions_dir: PathBuf,
    pub status_dir: PathBuf,
    pub signals_dir: PathBuf,
    #[serde(skip)]
    pub max_manual_duration_secs: u64,
    #[serde(skip)]
    pub status_update_interval: Duration,
    #[serde(skip)]
    pub file_write_delay_ms: u64,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            recordings_dir: PathBuf::from("storage/recordings"),
            transcriptions_dir: PathBuf::from("storage/transcriptions"),
            status_dir: PathBuf::from("storage/status"),
            signals_dir: PathBuf::from("storage/signals"),
            max_manual_duration_secs: 7200, // 2 hours
            status_update_interval: Duration::from_secs(1),
            file_write_delay_ms: 500,
        }
    }
}

impl RecorderConfig {
    pub fn new() -> Self {
        // Try to load from config file, otherwise use default
        Self::load().unwrap_or_default()
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.recordings_dir)?;
        std::fs::create_dir_all(&self.transcriptions_dir)?;
        std::fs::create_dir_all(&self.status_dir)?;
        std::fs::create_dir_all(&self.signals_dir)?;
        Ok(())
    }

    fn get_config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("audio-recorder-manager");

        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir.join("recorder_config.json"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let mut config: RecorderConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        // Set non-serialized fields
        config.max_manual_duration_secs = 7200;
        config.status_update_interval = Duration::from_secs(1);
        config.file_write_delay_ms = 500;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_file_path()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}
