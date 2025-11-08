mod config;
mod devices;
mod domain;
mod error;
mod recorder;
mod status;
mod wasapi_loopback;

use anyhow::Context;
use chrono::Local;
use config::RecorderConfig;
use devices::DeviceManager;
use domain::{AudioFormat, RecordingDuration, RecordingSession};
use error::Result;
#[cfg(not(windows))]
use recorder::AudioRecorder;
use recorder::{convert_wav_to_m4a, RecordingQuality};
use status::{JsonFileObserver, RecordingResult, RecordingStatus, StatusObserver};
use wasapi_loopback::windows_loopback::WasapiLoopbackRecorder;
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Match Python CLI interface: command [args...]
    if args.len() > 1 {
        let command = &args[1];

        match command.as_str() {
            "record" => {
                // Initialize config
                let config = RecorderConfig::new();

                // Parse: record <duration> [format] [quality]
                // Validate duration parameter
                let duration_secs: i64 = if args.len() > 2 {
                    match args[2].parse::<i64>() {
                        Ok(d) if d == -1 || d > 0 => d,
                        Ok(d) => {
                            eprintln!("Error: Duration must be -1 (manual mode) or a positive number, got: {}", d);
                            std::process::exit(1);
                        }
                        Err(_) => {
                            eprintln!("Error: Invalid duration '{}'. Must be a number.", args[2]);
                            std::process::exit(1);
                        }
                    }
                } else {
                    30
                };

                let duration = match RecordingDuration::from_secs(duration_secs, config.max_manual_duration_secs) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                };

                // Validate format parameter
                let audio_format = if args.len() > 3 {
                    match AudioFormat::from_str(&args[3]) {
                        Ok(fmt) => fmt,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    AudioFormat::Wav
                };

                // Parse quality parameter (optional)
                let quality = if args.len() > 4 {
                    let q = args[4].to_lowercase();
                    match q.as_str() {
                        "quick" => RecordingQuality::quick(),
                        "standard" => RecordingQuality::standard(),
                        "professional" => RecordingQuality::professional(),
                        "high" => RecordingQuality::high(),
                        _ => {
                            eprintln!("Error: Invalid quality '{}'. Options: quick, standard, professional, high", args[4]);
                            std::process::exit(1);
                        }
                    }
                } else {
                    RecordingQuality::professional() // Default quality
                };

                quick_record(duration, audio_format, quality, config).await?;
                return Ok(());
            }
            "status" => {
                let result = system_status().await?;
                println!("{}", serde_json::to_string(&result)?);
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command: {}", command);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    // Interactive mode
    print_usage();
    Ok(())
}

async fn quick_record(
    duration: RecordingDuration,
    audio_format: AudioFormat,
    quality: RecordingQuality,
    config: RecorderConfig,
) -> Result<serde_json::Value> {
    env_logger::init();

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

    Ok(result)
}

async fn record_worker(
    session: RecordingSession,
    config: RecorderConfig,
) -> Result<()> {
    let filepath = config.recordings_dir.join(session.temp_filename());
    let observer = JsonFileObserver::new(config.status_dir.clone());

    let effective_duration = session.duration.effective_duration();

    // Use WASAPI loopback on Windows
    #[cfg(windows)]
    {
        log::info!("Starting WASAPI loopback recording: {} ({} seconds)", session.temp_filename(), effective_duration);

        // Create recorder
        let recorder = WasapiLoopbackRecorder::new(filepath.clone())?;
        let start_time = std::time::Instant::now();

        // Write initial status
        observer.on_progress(RecordingStatus {
            session_id: session.id.as_str().to_string(),
            filename: session.temp_filename(),
            duration: effective_duration,
            elapsed: 0,
            progress: 0,
            quality: session.quality.name.clone(),
            device: "Default Output (WASAPI Loopback)".to_string(),
            sample_rate: recorder.get_sample_rate(),
            channels: recorder.get_channels(),
            frames_captured: recorder.get_frames_captured(),
            has_audio: recorder.has_audio_detected(),
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
            let stop_signal = config.signals_dir.join(format!("{}.stop", session.id.as_str()));
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

            // Print progress to terminal
            eprintln!(
                "[Recording] Progress: {}% | Elapsed: {}s / {}s | Frames: {} | Audio: {}",
                progress,
                elapsed,
                effective_duration,
                recorder.get_frames_captured(),
                if recorder.has_audio_detected() { "Yes" } else { "No" }
            );

            // Update status file
            observer.on_progress(RecordingStatus {
                session_id: session.id.as_str().to_string(),
                filename: session.temp_filename(),
                duration: effective_duration,
                elapsed,
                progress,
                quality: session.quality.name.clone(),
                device: "Default Output (WASAPI Loopback)".to_string(),
                sample_rate: recorder.get_sample_rate(),
                channels: recorder.get_channels(),
                frames_captured: recorder.get_frames_captured(),
                has_audio: recorder.has_audio_detected(),
                status: "recording".to_string(),
            })?;
        }

        // Stop recording
        recorder.stop()?;
        log::info!("Recording completed: {:?}", filepath);
        eprintln!("[Recording] Completed successfully!");

        // Wait a moment for file to be fully written
        tokio::time::sleep(Duration::from_millis(config.file_write_delay_ms)).await;
    }

    #[cfg(not(windows))]
    {
        // Fallback to CPAL for non-Windows platforms
        let device_manager = DeviceManager::new()
            .context("Failed to create device manager")?;

        let device = device_manager.get_best_recording_device()
            .context("Failed to get recording device")?;

        let device_raw = device.device()
            .context("Device not available")?
            .clone();

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
                config.status_dir.clone()
            )
            .await?;

        log::info!("Recording started: {} ({} seconds)", session.temp_filename(), effective_duration);
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
        filename: final_filepath.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string(),
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

async fn system_status() -> Result<serde_json::Value> {
    let device_manager = DeviceManager::new()
        .context("Failed to create device manager")?;

    let devices = device_manager.list_devices()?;

    let device_list: Vec<_> = devices
        .iter()
        .map(|d| {
            json!({
                "name": d.name,
                "is_default_input": d.is_default_input,
                "is_default_output": d.is_default_output,
            })
        })
        .collect();

    Ok(json!({
        "status": "success",
        "data": {
            "devices_count": devices.len(),
            "devices": device_list
        }
    }))
}

fn print_usage() {
    println!("============================================================");
    println!("Audio Recorder Manager - Rust Edition");
    println!("============================================================");
    println!();
    println!("Usage:");
    println!("  audio-recorder-manager record <duration> [format]");
    println!("  audio-recorder-manager status");
    println!();
    println!("Examples:");
    println!("  audio-recorder-manager record 30 wav    # Record for 30 seconds");
    println!("  audio-recorder-manager record -1 wav    # Manual mode (stop with signal)");
    println!("  audio-recorder-manager status           # Show system audio devices");
    println!();
}
