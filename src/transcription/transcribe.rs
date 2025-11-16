use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub success: bool,
    pub transcript_file: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionStatus {
    pub step: String,
    pub step_number: i32,
    pub total_steps: i32,
    pub progress: i32,
    pub message: String,
}

// Gemini API structures
#[derive(Debug, Deserialize)]
struct FileUploadResponse {
    file: UploadedFile,
}

#[derive(Debug, Deserialize)]
struct UploadedFile {
    name: String,
    uri: String,
    state: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Debug, Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    FileData { file_data: FileReference },
}

#[derive(Debug, Serialize)]
struct FileReference {
    mime_type: String,
    file_uri: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

/// Write transcription status to file for UI monitoring
fn write_status(session_id: &str, step: i32, total_steps: i32, step_name: &str, message: &str) -> Result<()> {
    let status_dir = get_status_dir()?;
    let status_file = status_dir.join(format!("{}.json", session_id));

    let progress = (step * 100) / total_steps;
    let status = TranscriptionStatus {
        step: step_name.to_string(),
        step_number: step,
        total_steps,
        progress,
        message: message.to_string(),
    };

    let json = serde_json::to_string_pretty(&status)?;
    fs::write(&status_file, json)?;
    Ok(())
}

fn get_status_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let status_dir = current_dir.join("storage").join("status");
    fs::create_dir_all(&status_dir)?;
    Ok(status_dir)
}

/// Transcribe an audio file using the Gemini API
pub async fn transcribe_audio(
    audio_file_path: &Path,
    api_key: &str,
    model: &str,
    prompt: &str,
    _optimize: bool,
    session_id: &str,
) -> Result<TranscriptionResult> {
    // Validate audio file exists
    if !audio_file_path.exists() {
        bail!("Audio file not found: {}", audio_file_path.display());
    }

    let output_file = audio_file_path.with_extension("md");

    // Determine MIME type
    let mime_type = match audio_file_path.extension().and_then(|e| e.to_str()) {
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        _ => bail!("Unsupported audio format: {:?}", audio_file_path.extension()),
    };

    log::info!("Starting transcription for: {}", audio_file_path.display());
    log::info!("Model: {}, MIME: {}", model, mime_type);

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    // Step 1: Read and encode audio file
    write_status(session_id, 1, 4, "reading", "Reading audio file...")?;
    log::info!("[1/4] Reading audio file...");

    let audio_data = fs::read(audio_file_path)
        .context("Failed to read audio file")?;
    let file_size_mb = audio_data.len() as f64 / (1024.0 * 1024.0);
    log::info!("File size: {:.2} MB", file_size_mb);

    // Step 2: Upload file to Gemini API
    write_status(session_id, 2, 4, "uploading", "Uploading file to Gemini API...")?;
    log::info!("[2/4] Uploading to Gemini API...");

    let upload_url = format!(
        "https://generativelanguage.googleapis.com/upload/v1beta/files?key={}",
        api_key
    );

    let upload_request = serde_json::json!({
        "file": {
            "mimeType": mime_type,
            "displayName": audio_file_path.file_name().and_then(|n| n.to_str()).unwrap_or("audio")
        }
    });

    // First, initiate the upload
    log::debug!("Initiating resumable upload to Gemini API...");
    let init_response = client
        .post(&upload_url)
        .header("X-Goog-Upload-Protocol", "resumable")
        .header("X-Goog-Upload-Command", "start")
        .header("X-Goog-Upload-Header-Content-Length", audio_data.len().to_string())
        .header("X-Goog-Upload-Header-Content-Type", mime_type)
        .json(&upload_request)
        .send()
        .await
        .context("Failed to initiate upload")?;

    let status = init_response.status();
    log::debug!("Upload initiation response status: {}", status);

    if !status.is_success() {
        let error_text = init_response.text().await?;
        log::error!("Upload initiation failed with status {}: {}", status, error_text);
        bail!("Upload initiation failed: {}", error_text);
    }

    let upload_session_url = init_response
        .headers()
        .get("X-Goog-Upload-URL")
        .context("No upload URL in response")?
        .to_str()?
        .to_string();

    log::debug!("Upload session URL obtained: {}", upload_session_url);
    log::info!("Uploading {} bytes of audio data...", audio_data.len());

    // Now upload the actual file data
    let upload_start = std::time::Instant::now();
    let upload_response = client
        .post(&upload_session_url)
        .header("X-Goog-Upload-Command", "upload, finalize")
        .header("X-Goog-Upload-Offset", "0")
        .header("Content-Length", audio_data.len().to_string())
        .header("Content-Type", mime_type)
        .body(audio_data)
        .send()
        .await
        .context("Failed to upload file")?;

    let upload_duration = upload_start.elapsed();
    let upload_status = upload_response.status();
    log::debug!("File upload response status: {} (took {:.2}s)", upload_status, upload_duration.as_secs_f64());

    if !upload_status.is_success() {
        let error_text = upload_response.text().await?;
        log::error!("File upload failed with status {}: {}", upload_status, error_text);
        bail!("File upload failed: {}", error_text);
    }

    let upload_result: FileUploadResponse = upload_response.json().await
        .context("Failed to parse upload response")?;

    log::info!("File uploaded successfully: {} (state: {})", upload_result.file.uri, upload_result.file.state);

    // Step 3: Wait for file processing (if needed)
    write_status(session_id, 3, 4, "processing", "Waiting for file processing...")?;
    log::info!("[3/4] Waiting for file processing...");

    let mut file_info = upload_result.file;
    let get_file_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
        file_info.name, api_key
    );

    // Poll until file is active
    let mut attempts = 0;
    let processing_start = std::time::Instant::now();

    while file_info.state == "PROCESSING" && attempts < 60 {
        log::debug!("Polling file status (attempt {}/60)...", attempts + 1);
        tokio::time::sleep(Duration::from_secs(2)).await;

        let response = client.get(&get_file_url).send().await?;
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            if let Some(state) = response_json["file"]["state"].as_str() {
                file_info.state = state.to_string();
                log::debug!("File state: {}", state);
            }
        }
        attempts += 1;
    }

    let processing_duration = processing_start.elapsed();

    if file_info.state == "FAILED" {
        log::error!("File processing failed after {:.2}s", processing_duration.as_secs_f64());
        bail!("File processing failed");
    }

    if attempts >= 60 {
        log::warn!("File processing timeout after {} attempts ({:.2}s)", attempts, processing_duration.as_secs_f64());
    }

    log::info!("File processing completed in {:.2}s, state: {}", processing_duration.as_secs_f64(), file_info.state);

    // Step 4: Generate transcription
    write_status(session_id, 4, 4, "transcribing", "Generating transcription...")?;
    log::info!("[4/4] Generating transcription...");

    let generate_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GenerateContentRequest {
        contents: vec![Content {
            parts: vec![
                Part::Text {
                    text: prompt.to_string(),
                },
                Part::FileData {
                    file_data: FileReference {
                        mime_type: file_info.mime_type.clone(),
                        file_uri: file_info.uri.clone(),
                    },
                },
            ],
        }],
        generation_config: GenerationConfig {
            temperature: 0.1,
            top_p: 0.95,
            top_k: 40,
            max_output_tokens: 8192,
        },
    };

    log::debug!("Sending content generation request to {} with file URI: {}", model, file_info.uri);
    log::debug!("Generation config: temp=0.1, top_p=0.95, top_k=40, max_tokens=8192");

    let generation_start = std::time::Instant::now();
    let generate_response = client
        .post(&generate_url)
        .json(&request)
        .send()
        .await
        .context("Failed to generate content")?;

    let generation_duration = generation_start.elapsed();
    let gen_status = generate_response.status();
    log::debug!("Content generation response status: {} (took {:.2}s)", gen_status, generation_duration.as_secs_f64());

    if !gen_status.is_success() {
        let error_text = generate_response.text().await?;
        log::error!("Content generation failed with status {}: {}", gen_status, error_text);
        log::error!("Generate URL was: {}", generate_url);
        bail!("Content generation failed: {}", error_text);
    }

    // Log raw response for debugging
    let response_text = generate_response.text().await?;
    log::debug!("Raw API response: {}", &response_text[..response_text.len().min(500)]);

    let result: GenerateContentResponse = serde_json::from_str(&response_text)
        .context("Failed to parse generation response")?;

    let transcript = result.candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .context("No transcript in response")?;

    log::info!("Transcription completed in {:.2}s, length: {} chars ({} words approx)",
        generation_duration.as_secs_f64(),
        transcript.len(),
        transcript.split_whitespace().count());

    // Save transcript to file
    log::debug!("Writing transcript to: {}", output_file.display());
    fs::write(&output_file, &transcript)
        .context("Failed to write transcript file")?;

    log::info!("Transcript saved successfully to: {}", output_file.display());

    Ok(TranscriptionResult {
        success: true,
        transcript_file: Some(output_file.to_string_lossy().to_string()),
        error: None,
    })
}

/// Read transcription status from status file
pub fn read_transcription_status(session_id: &str) -> Result<Option<TranscriptionStatus>> {
    let status_dir = get_status_dir()?;
    let status_file = status_dir.join(format!("{}.json", session_id));

    if !status_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&status_file)?;
    let status: TranscriptionStatus = serde_json::from_str(&content)?;
    Ok(Some(status))
}
