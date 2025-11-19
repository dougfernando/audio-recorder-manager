use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct RecorderConfig {
    pub storage_dir: PathBuf,
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

#[derive(Deserialize)]
struct TempConfig {
    storage_dir: Option<PathBuf>,
    recordings_dir: Option<PathBuf>,
}


impl Default for RecorderConfig {
    fn default() -> Self {
        let storage_base = Self::get_workspace_storage_dir();
        Self::from_storage_dir(storage_base)
    }
}

impl RecorderConfig {
    /// Get the workspace root storage directory
    /// This ensures both CLI and Tauri use the same storage location
    fn get_workspace_storage_dir() -> PathBuf {
        // Try to find workspace root by looking for Cargo.toml with [workspace]
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Check if we're in a workspace subdirectory
        let mut search_dir = current_dir.clone();
        for _ in 0..3 {
            let cargo_toml = search_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    if content.contains("[workspace]") {
                        return search_dir.join("storage");
                    }
                }
            }
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        // Fallback to current directory
        current_dir.join("storage")
    }

    pub fn from_storage_dir(storage_dir: PathBuf) -> Self {
        Self {
            recordings_dir: storage_dir.join("recordings"),
            transcriptions_dir: storage_dir.join("transcriptions"),
            status_dir: storage_dir.join("status"),
            signals_dir: storage_dir.join("signals"),
            storage_dir,
            max_manual_duration_secs: 7200,
            status_update_interval: Duration::from_secs(1),
            file_write_delay_ms: 500,
        }
    }


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

        let temp_config: TempConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        let storage_dir = temp_config.storage_dir.or_else(|| {
            temp_config.recordings_dir.and_then(|p| p.parent().map(|path| path.to_path_buf()))
        }).unwrap_or_else(Self::get_workspace_storage_dir);
        
        let mut config = Self::from_storage_dir(storage_dir);


        // Set non-serialized fields
        config.max_manual_duration_secs = 7200;
        config.status_update_interval = Duration::from_secs(1);
        config.file_write_delay_ms = 500;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        #[derive(Serialize)]
        struct ConfigToSave<'a> {
            storage_dir: &'a PathBuf,
        }

        let to_save = ConfigToSave {
            storage_dir: &self.storage_dir,
        };

        let config_path = Self::get_config_file_path()?;
        let content = serde_json::to_string_pretty(&to_save)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}
