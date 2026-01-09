use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub optimize_audio: bool,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gemini-2.5-flash".to_string(),
            prompt: r#"Please process the attached audio file and provide the following content in markdown format:

            **Topics Discussed:**

            *   Detect the language spoken in the audio.
            *   Analyze the whole audio in details.
            *   Identify the all subjects, decisions, and action items, including details, examples, numbers and names mentioned.
            *   Organize the content into categories with clear headings for each topic; if something is hard to classify, add to "Others section".
            *   Describe the topics in the same language as identified in the audio as long it is Spanish, Portuguese or English; otherwise, use English.
            *   Ensure absolutely no information is lost from the audio, better add irrelevant content in the audio than lose nuances.

            Your entire response should be a single markdown document and absolutely not content should be lost."#.to_string(),
            optimize_audio: false,
        }
    }
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let app_data =
        std::env::var("APPDATA").context("Failed to get APPDATA environment variable")?;
    let config_dir = PathBuf::from(app_data).join("audio-recorder-manager");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("transcription_config.json"))
}

pub fn load_config() -> Result<TranscriptionConfig> {
    let config_path = get_config_file_path()?;

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(TranscriptionConfig::default());
    }

    let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
    let config: TranscriptionConfig =
        serde_json::from_str(&content).context("Failed to parse config file")?;

    Ok(config)
}

pub fn save_config(config: &TranscriptionConfig) -> Result<()> {
    let config_path = get_config_file_path()?;
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&config_path, content).context("Failed to write config file")?;
    Ok(())
}
