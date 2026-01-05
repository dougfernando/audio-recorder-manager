use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::Instrument;

use crate::config::RecorderConfig;
#[cfg(not(windows))]
use crate::devices::DeviceManager;
use crate::domain::{AudioFormat, RecordingDuration, RecordingSession};
use crate::error::Result;
use crate::output::UserOutput;
#[cfg(not(windows))]
use crate::recorder::AudioRecorder;
use crate::recorder::{merge_audio_streams_smart, RecordingQuality};
#[cfg(not(windows))]
use crate::recorder::convert_wav_to_m4a;
use crate::status::{JsonFileObserver, RecordingResult, RecordingStatus, StatusObserver};
use crate::wasapi_loopback::windows_loopback::WasapiLoopbackRecorder;
use crate::wasapi_microphone::windows_microphone::WasapiMicrophoneRecorder;

/// Execute the record command
pub async fn execute(
    duration: RecordingDuration,
    audio_format: AudioFormat,
    quality: RecordingQuality,
    config: RecorderConfig,
) -> Result<()> {
    execute_with_output(duration, audio_format, quality, config, UserOutput::new()).await
}

/// Execute the record command with custom output
pub async fn execute_with_output(
    duration: RecordingDuration,
    audio_format: AudioFormat,
    quality: RecordingQuality,
    config: RecorderConfig,
    output: UserOutput,
) -> Result<()> {
    // Note: Logging is initialized by the binary (CLI or Tauri), not here

    // Create recording session
    let session = RecordingSession::new(audio_format, quality.clone(), duration);

    // Ensure directories exist
    config.ensure_directories()?;

    // Return success result with final filename
    let result = json!({
        "status": "success",
        "data": {
            "session_id": session.id.as_str(),
            "file_path": config.recordings_dir.join(session.filename()).to_string_lossy(),
            "filename": session.filename(),
            "duration": duration.to_api_value(),
            "quality": quality.name,
            "message": "Recording started successfully"
        }
    });

    // Print JSON immediately (like Python version does with print and flush)
    println!("{}", serde_json::to_string(&result)?);

    // Now do the actual recording (this blocks, keeping process alive)
    record_worker(session, config, output).await?;

    Ok(())
}

/// Wait for a file to be fully written and readable
/// Checks both that the file exists and that its size hasn't changed
async fn wait_for_file_ready(path: &std::path::PathBuf, timeout_ms: u64) -> Result<()> {
    let start = tokio::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    loop {
        // Check if file exists and is readable
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();

            // Wait a bit more and check size again
            tokio::time::sleep(Duration::from_millis(50)).await;

            if let Ok(metadata2) = std::fs::metadata(path) {
                // If size hasn't changed, file is done writing
                if size == metadata2.len() && size > 0 {
                    tracing::debug!("✓ File ready: {:?} ({} bytes)", path, size);
                    return Ok(());
                }
            }
        }

        if start.elapsed() > timeout {
            tracing::warn!("⚠ Timeout waiting for file: {:?} (continuing anyway)", path);
            return Ok(()); // Continue anyway, file might be ready
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn record_worker(
    session: RecordingSession,
    config: RecorderConfig,
    output: UserOutput,
) -> Result<()> {
    // Capture values for the span before moving session
    let session_id = session.id.clone();
    let format = session.format;
    let duration_secs = session.duration.effective_duration();

    async move {
        let filepath = config.recordings_dir.join(session.temp_filename());
        let observer = Arc::new(JsonFileObserver::new(config.status_dir.clone()));
        let effective_duration = session.duration.effective_duration();
        let mut final_filepath = filepath.clone();

        // Use WASAPI dual-channel recording on Windows (loopback + microphone)
        #[cfg(windows)]
        {
        tracing::info!(
            "Starting WASAPI dual-channel recording: {} ({} seconds)",
            session.temp_filename(),
            effective_duration
        );

        // Create temporary filenames for separate recordings
        let loopback_temp = config.recordings_dir.join(format!(
            "{}_loopback.wav",
            session.id.as_str()
        ));
        let mic_temp = config.recordings_dir.join(format!(
            "{}_mic.wav",
            session.id.as_str()
        ));

        // Initialize loopback recorder (system audio) - REQUIRED
        let loopback_recorder = WasapiLoopbackRecorder::new(loopback_temp.clone())?;

        // Get loopback sample rate to match microphone
        let target_sample_rate = loopback_recorder.get_sample_rate();
        tracing::info!("System audio sample rate: {} Hz - will match microphone to this", target_sample_rate);

        // Initialize microphone recorder with matched sample rate
        let mic_recorder = match WasapiMicrophoneRecorder::new(mic_temp.clone(), target_sample_rate) {
            Ok(recorder) => {
                tracing::info!("Microphone recorder initialized successfully with matched sample rate");
                Some(recorder)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize microphone recorder: {}. Continuing with loopback only.", e);
                output.warning("Microphone unavailable - recording system audio only");
                None
            }
        };

        let start_time = std::time::Instant::now();

        // Write initial status
        observer.on_progress(RecordingStatus {
            session_id: session.id.as_str().to_string(),
            filename: session.temp_filename(),
            duration: effective_duration,
            elapsed: 0,
            progress: 0,
            quality: session.quality.name.clone(),
            device: if mic_recorder.is_some() {
                "Dual-Channel (System + Microphone)".to_string()
            } else {
                "System Audio Only (WASAPI Loopback)".to_string()
            },
            sample_rate: loopback_recorder.get_sample_rate(),
            channels: 2, // Always stereo output
            frames_captured: loopback_recorder.get_frames_captured(),
            has_audio: loopback_recorder.has_audio_detected(),
            status: "recording".to_string(),
            // Per-channel data (initial state)
            loopback_frames: Some(loopback_recorder.get_frames_captured()),
            loopback_has_audio: Some(loopback_recorder.has_audio_detected()),
            mic_frames: mic_recorder.as_ref().map(|m| m.get_frames_captured()),
            mic_has_audio: mic_recorder.as_ref().map(|m| m.has_audio_detected()),
            // FFmpeg progress (not applicable during recording)
            ffmpeg_progress: None,
            processing_speed: None,
        })?;

        // Update status every second
        let update_interval = config.status_update_interval;
        loop {
            tokio::time::sleep(update_interval).await;

            let elapsed = start_time.elapsed().as_secs();

            // Check for stop conditions
            if elapsed >= effective_duration {
                break;
            }

            // Check for cancel signal (skip processing)
            let cancel_signal = config
                .signals_dir
                .join(format!("{}.cancel", session.id.as_str()));
            if cancel_signal.exists() {
                tracing::info!("Cancel signal received for session {}", session.id);
                let _ = std::fs::remove_file(cancel_signal);

                // Stop both recorders
                loopback_recorder.stop()?;
                if let Some(ref mic) = mic_recorder {
                    mic.stop()?;
                }

                // Cleanup temporary files
                let _ = std::fs::remove_file(&loopback_temp);
                let _ = std::fs::remove_file(&mic_temp);
                tracing::info!("Recording cancelled, temporary files cleaned up");

                // Write cancelled status
                observer.on_complete(RecordingResult {
                    session_id: session.id.as_str().to_string(),
                    filename: "".to_string(),
                    file_path: None,
                    duration: 0,
                    file_size_mb: "0 MB".to_string(),
                    format: session.format.to_string(),
                    codec: "".to_string(),
                    status: "cancelled".to_string(),
                    message: "Recording cancelled by user".to_string(),
                })?;

                output.warning("Recording cancelled by user");
                return Ok(());
            }

            // Check for stop signal
            let stop_signal = config
                .signals_dir
                .join(format!("{}.stop", session.id.as_str()));
            if stop_signal.exists() {
                tracing::info!("Stop signal received for session {}", session.id);
                let _ = std::fs::remove_file(stop_signal);
                break;
            }

            // Calculate progress
            let progress = if effective_duration > 0 {
                ((elapsed as f64 / effective_duration as f64) * 100.0).min(100.0) as u8
            } else {
                0
            };

            // Print progress to terminal (dual-channel format)
            if let Some(ref mic) = mic_recorder {
                output.prefixed(
                    "Recording",
                    &format!(
                        "Progress: {}% | Elapsed: {}s / {}s | Loopback: {} frames ({}) | Mic: {} frames ({})",
                        progress,
                        elapsed,
                        effective_duration,
                        loopback_recorder.get_frames_captured(),
                        if loopback_recorder.has_audio_detected() { "Audio" } else { "Silent" },
                        mic.get_frames_captured(),
                        if mic.has_audio_detected() { "Audio" } else { "Silent" }
                    )
                );
            } else {
                output.prefixed(
                    "Recording",
                    &format!(
                        "Progress: {}% | Elapsed: {}s / {}s | Loopback: {} frames ({})",
                        progress,
                        elapsed,
                        effective_duration,
                        loopback_recorder.get_frames_captured(),
                        if loopback_recorder.has_audio_detected() { "Audio" } else { "Silent" }
                    )
                );
            }

            // Update status file
            observer.on_progress(RecordingStatus {
                session_id: session.id.as_str().to_string(),
                filename: session.temp_filename(),
                duration: effective_duration,
                elapsed,
                progress,
                quality: session.quality.name.clone(),
                device: if mic_recorder.is_some() {
                    "Dual-Channel (System + Microphone)".to_string()
                } else {
                    "System Audio Only (WASAPI Loopback)".to_string()
                },
                sample_rate: loopback_recorder.get_sample_rate(),
                channels: 2, // Always stereo output
                frames_captured: loopback_recorder.get_frames_captured(),
                has_audio: loopback_recorder.has_audio_detected(),
                status: "recording".to_string(),
                // Per-channel data
                loopback_frames: Some(loopback_recorder.get_frames_captured()),
                loopback_has_audio: Some(loopback_recorder.has_audio_detected()),
                mic_frames: mic_recorder.as_ref().map(|m| m.get_frames_captured()),
                mic_has_audio: mic_recorder.as_ref().map(|m| m.has_audio_detected()),
                // FFmpeg progress (not applicable during recording)
                ffmpeg_progress: None,
                processing_speed: None,
            })?;
        }

        // Stop both recorders
        loopback_recorder.stop()?;
        if let Some(ref mic) = mic_recorder {
            mic.stop()?;
        }

        tracing::info!("Recording completed, starting post-processing");
        output.success("Recording completed successfully!");

        // Wait for temporary files to be fully written and readable before processing
        tracing::info!("⏳ Waiting for temporary files to be ready...");
        let _ = wait_for_file_ready(&loopback_temp, 2000).await;
        let _ = wait_for_file_ready(&mic_temp, 2000).await;
        tracing::info!("✓ Temporary files are ready for processing");

        // Determine total steps based on format
        let total_steps = if matches!(session.format, AudioFormat::M4a) {
            4  // M4A: Analyze -> Merge -> Encode -> Finalize
        } else {
            3  // WAV: Analyze -> Merge -> Finalize
        };

        // Stage 1: Analyzing Audio
        let loopback_has_audio = loopback_recorder.has_audio_detected();
        let mic_has_audio = mic_recorder.as_ref().map(|m| m.has_audio_detected()).unwrap_or(false);

        let analysis_message = if loopback_has_audio && mic_has_audio {
            "Detected system audio and microphone"
        } else if loopback_has_audio {
            "Detected system audio only"
        } else if mic_has_audio {
            "Detected microphone only"
        } else {
            "No audio detected"
        };

        tracing::info!("📝 Stage 1/{}: Analyzing Audio - {}", total_steps, analysis_message);
        let _ = observer.write_processing_status_v2(
            session.id.as_str(),
            analysis_message,
            Some(1),
            Some(total_steps),
            Some("analyzing"),
            None,
            Some(session.duration.to_api_value() as u64),
        );

        tracing::info!("🚀 Post-processing started");

        // Merge audio channels (and encode to M4A if requested) in a span for better tracing
        {
            let loopback_has_audio_flag = loopback_recorder.has_audio_detected();
            let mic_has_audio_flag = mic_recorder.as_ref().map(|m| m.has_audio_detected()).unwrap_or(false);

            async {
                // Determine output file based on format
                let merge_output_path = if matches!(session.format, AudioFormat::M4a) {
                    output.prefixed("Processing", "Merging and encoding to M4A...");
                    filepath.with_extension("m4a")
                } else {
                    output.prefixed("Processing", "Merging audio channels...");
                    filepath.clone()
                };

                // Small delay to ensure status file was written before merge starts
                tokio::time::sleep(Duration::from_millis(200)).await;

                // Get audio detection flags
                let loopback_has_audio = loopback_recorder.has_audio_detected();
                let mic_has_audio = mic_recorder.as_ref()
                    .map(|m| m.has_audio_detected())
                    .unwrap_or(false);

                // Stage 2 will be emitted inside merge_audio_streams_smart for better granularity
                // Merge audio streams using FFmpeg (with direct M4A encoding if requested)
                merge_audio_streams_smart(
                    &loopback_temp,
                    &mic_temp,
                    &merge_output_path,
                    loopback_has_audio,
                    mic_has_audio,
                    &session.quality,
                    session.format,
                    Some(session.id.as_str()),
                    Some(observer.clone()),
                    total_steps,
                )
                .await?;

                tracing::info!("Audio processing completed: {:?}", merge_output_path);
                output.success("Audio processing completed!");
                Ok::<(), crate::error::RecorderError>(())
            }
            .instrument(tracing::info_span!(
                "merge_audio",
                loopback_has_audio = loopback_has_audio_flag,
                mic_has_audio = mic_has_audio_flag
            ))
            .await?;
        }

        // Cleanup temporary files
        let _ = std::fs::remove_file(&loopback_temp);
        let _ = std::fs::remove_file(&mic_temp);
        tracing::info!("Temporary files cleaned up");

        // Update filepath to point to the merged output
        // For M4A, we already have the final file (merge+encode was done in one pass)
        // For WAV, filepath is already correct
        final_filepath = if matches!(session.format, AudioFormat::M4a) {
            filepath.with_extension("m4a")
        } else {
            filepath.clone()
        };
    }

    #[cfg(not(windows))]
    {
        // Fallback to CPAL for non-Windows platforms
        let device_manager =
            DeviceManager::new().context("Failed to create device manager")?;

        let device = device_manager
            .get_best_recording_device()
            .context("Failed to get recording device")?;

        let device_raw = device.device().context("Device not available")?.clone();

        let recorder = AudioRecorder::new(
            device_raw,
            device.name.clone(),
            config.recordings_dir.clone(),
        )?;

        let handle = recorder
            .start_recording(
                &session.temp_filename(),
                Some(effective_duration),
                session.id.as_str().to_string(),
                config.status_dir.clone(),
            )
            .await?;

        tracing::info!(
            "Recording started: {} ({} seconds)",
            session.temp_filename(),
            effective_duration
        );
        handle.write_status()?;

        let update_interval = config.status_update_interval;
        loop {
            tokio::time::sleep(update_interval).await;

            if handle.should_stop() {
                break;
            }

            handle.write_status()?;
        }

        let _ = handle.stop().await?;
        tracing::info!("Recording completed: {:?}", filepath);
    }

    // Convert to M4A if requested (only for non-Windows platforms)
    // On Windows, M4A encoding is done during merge (one-pass optimization)
    // On non-Windows, we need a separate conversion step
    #[cfg(not(windows))]
    if matches!(session.format, AudioFormat::M4a) {
        tracing::info!("Converting WAV to M4A (part of post-processing)...");
        output.prefixed("Processing", "Encoding to M4A...");

        let m4a_path = filepath.with_extension("m4a");

        match convert_wav_to_m4a(&filepath, &m4a_path).await {
            Ok(_) => {
                tracing::info!("Successfully converted to M4A: {:?}", m4a_path);
                output.success("M4A encoding complete!");
                // Delete temporary WAV file
                if let Err(e) = std::fs::remove_file(&filepath) {
                    tracing::warn!("Failed to delete temporary WAV file: {}", e);
                }
                final_filepath = m4a_path;
            }
            Err(e) => {
                tracing::error!("Failed to convert to M4A: {}. Keeping WAV file.", e);
                output.warning("Failed to convert to M4A. Keeping WAV file.");
            }
        }
    }

    // Write final status
    let file_size_mb = if final_filepath.exists() {
        let size = std::fs::metadata(&final_filepath)?.len();
        (size as f64) / (1024.0 * 1024.0)
    } else {
        0.0
    };

    observer.on_complete(RecordingResult {
        status: "completed".to_string(),
        session_id: session.id.as_str().to_string(),
        filename: final_filepath
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        file_path: Some(final_filepath.to_string_lossy().to_string()),
        duration: session.duration.to_api_value(),
        file_size_mb: format!("{:.2}", file_size_mb),
        format: session.format.to_string(),
        codec: session.format.codec().to_string(),
        message: match session.format {
            AudioFormat::M4a => "Recording converted to M4A successfully".to_string(),
            AudioFormat::Wav => "Recording completed successfully".to_string(),
        },
    })?;

        Ok(())
    }
    .instrument(tracing::info_span!(
        "recording_session",
        session_id = %session_id,
        format = ?format,
        duration_secs = duration_secs
    ))
    .await
}
