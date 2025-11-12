use serde_json::json;
use std::time::Duration;

use crate::config::RecorderConfig;
#[cfg(not(windows))]
use crate::devices::DeviceManager;
use crate::domain::{AudioFormat, RecordingDuration, RecordingSession};
use crate::error::Result;
#[cfg(not(windows))]
use crate::recorder::AudioRecorder;
use crate::recorder::{convert_wav_to_m4a, merge_audio_streams_smart, RecordingQuality};
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
    let _ = env_logger::try_init();

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
    record_worker(session, config).await?;

    Ok(())
}

async fn record_worker(session: RecordingSession, config: RecorderConfig) -> Result<()> {
    let filepath = config.recordings_dir.join(session.temp_filename());
    let observer = JsonFileObserver::new(config.status_dir.clone());

    let effective_duration = session.duration.effective_duration();

    // Use WASAPI dual-channel recording on Windows (loopback + microphone)
    #[cfg(windows)]
    {
        log::info!(
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
        log::info!("System audio sample rate: {} Hz - will match microphone to this", target_sample_rate);

        // Initialize microphone recorder with matched sample rate
        let mic_recorder = match WasapiMicrophoneRecorder::new(mic_temp.clone(), target_sample_rate) {
            Ok(recorder) => {
                log::info!("Microphone recorder initialized successfully with matched sample rate");
                Some(recorder)
            }
            Err(e) => {
                log::warn!("Failed to initialize microphone recorder: {}. Continuing with loopback only.", e);
                eprintln!("[Warning] Microphone unavailable - recording system audio only");
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

            // Check for stop signal
            let stop_signal = config
                .signals_dir
                .join(format!("{}.stop", session.id.as_str()));
            if stop_signal.exists() {
                log::info!("Stop signal received for session {}", session.id);
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
                eprintln!(
                    "[Recording] Progress: {}% | Elapsed: {}s / {}s | Loopback: {} frames ({}) | Mic: {} frames ({})",
                    progress,
                    elapsed,
                    effective_duration,
                    loopback_recorder.get_frames_captured(),
                    if loopback_recorder.has_audio_detected() { "Audio" } else { "Silent" },
                    mic.get_frames_captured(),
                    if mic.has_audio_detected() { "Audio" } else { "Silent" }
                );
            } else {
                eprintln!(
                    "[Recording] Progress: {}% | Elapsed: {}s / {}s | Loopback: {} frames ({})",
                    progress,
                    elapsed,
                    effective_duration,
                    loopback_recorder.get_frames_captured(),
                    if loopback_recorder.has_audio_detected() { "Audio" } else { "Silent" }
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
            })?;
        }

        // Stop both recorders
        loopback_recorder.stop()?;
        if let Some(ref mic) = mic_recorder {
            mic.stop()?;
        }

        log::info!("Recording completed, starting merge process");
        eprintln!("[Recording] Completed successfully!");
        eprintln!("[Merging] Merging audio channels...");

        // Wait a moment for files to be fully written
        tokio::time::sleep(Duration::from_millis(config.file_write_delay_ms)).await;

        // Get audio detection flags
        let loopback_has_audio = loopback_recorder.has_audio_detected();
        let mic_has_audio = mic_recorder.as_ref()
            .map(|m| m.has_audio_detected())
            .unwrap_or(false);

        // Merge audio streams using FFmpeg
        merge_audio_streams_smart(
            &loopback_temp,
            &mic_temp,
            &filepath,
            loopback_has_audio,
            mic_has_audio,
            &session.quality,
        )
        .await?;

        log::info!("Audio merge completed: {:?}", filepath);
        eprintln!("[Merging] Successfully merged audio channels!");

        // Cleanup temporary files
        let _ = std::fs::remove_file(&loopback_temp);
        let _ = std::fs::remove_file(&mic_temp);
        log::info!("Temporary files cleaned up");
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

        log::info!(
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
        log::info!("Recording completed: {:?}", filepath);
    }

    // Convert to M4A if requested
    let mut final_filepath = filepath.clone();
    if matches!(session.format, AudioFormat::M4a) {
        log::info!("Converting WAV to M4A...");
        eprintln!("[Converting] WAV to M4A format...");
        let m4a_path = filepath.with_extension("m4a");

        match convert_wav_to_m4a(&filepath, &m4a_path).await {
            Ok(_) => {
                log::info!("Successfully converted to M4A: {:?}", m4a_path);
                eprintln!("[Converting] Successfully converted to M4A format!");
                // Delete temporary WAV file
                if let Err(e) = std::fs::remove_file(&filepath) {
                    log::warn!("Failed to delete temporary WAV file: {}", e);
                }
                final_filepath = m4a_path;
            }
            Err(e) => {
                log::error!("Failed to convert to M4A: {}. Keeping WAV file.", e);
                eprintln!("[Converting] Failed to convert to M4A. Keeping WAV file.");
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
