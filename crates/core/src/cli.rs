use std::str::FromStr;

use crate::commands;
use crate::config::RecorderConfig;
use crate::domain::{AudioFormat, RecordingDuration};
use crate::error::Result;
use crate::output::UserOutput;
use crate::recorder::RecordingQuality;

pub async fn run(args: Vec<String>) -> Result<()> {
    let output = UserOutput::new();

    if args.len() <= 1 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "record" => handle_record_command(&args, &output).await,
        "status" => commands::status::execute().await,
        "stop" => handle_stop_command(&args, &output).await,
        "recover" => handle_recover_command(&args, &output).await,
        _ => {
            output.error(&format!("Unknown command: {}", command));
            print_usage();
            std::process::exit(1);
        }
    }
}

async fn handle_record_command(args: &[String], output: &UserOutput) -> Result<()> {
    let config = RecorderConfig::new();

    // Parse duration
    let duration_secs: i64 = if args.len() > 2 {
        match args[2].parse::<i64>() {
            Ok(d) if d == -1 || d > 0 => d,
            Ok(d) => {
                output.error(&format!(
                    "Duration must be -1 (manual mode) or a positive number, got: {}",
                    d
                ));
                std::process::exit(1);
            }
            Err(_) => {
                output.error(&format!("Invalid duration '{}'. Must be a number.", args[2]));
                std::process::exit(1);
            }
        }
    } else {
        30
    };

    let duration = match RecordingDuration::from_secs(duration_secs, config.max_manual_duration_secs)
    {
        Ok(d) => d,
        Err(e) => {
            output.error(&e.to_string());
            std::process::exit(1);
        }
    };

    // Parse format
    let audio_format = if args.len() > 3 {
        match AudioFormat::from_str(&args[3]) {
            Ok(fmt) => fmt,
            Err(e) => {
                output.error(&e.to_string());
                std::process::exit(1);
            }
        }
    } else {
        AudioFormat::Wav
    };

    // Parse quality
    let quality = if args.len() > 4 {
        let q = args[4].to_lowercase();
        match q.as_str() {
            "quick" => RecordingQuality::quick(),
            "standard" => RecordingQuality::standard(),
            "professional" => RecordingQuality::professional(),
            "high" => RecordingQuality::high(),
            _ => {
                output.error(&format!(
                    "Invalid quality '{}'. Options: quick, standard, professional, high",
                    args[4]
                ));
                std::process::exit(1);
            }
        }
    } else {
        RecordingQuality::professional()
    };

    commands::record::execute_with_output(duration, audio_format, quality, config, output.clone()).await
}

async fn handle_stop_command(args: &[String], _output: &UserOutput) -> Result<()> {
    let config = RecorderConfig::new();
    let session_id = if args.len() > 2 {
        Some(args[2].clone())
    } else {
        None
    };

    commands::stop::execute(session_id, config).await
}

async fn handle_recover_command(args: &[String], output: &UserOutput) -> Result<()> {
    let config = RecorderConfig::new();

    // Parse optional session_id
    let session_id = if args.len() > 2 && !args[2].starts_with("--") {
        Some(args[2].clone())
    } else {
        None
    };

    // Parse optional format
    let target_format = if args.len() > 3 {
        match AudioFormat::from_str(&args[3]) {
            Ok(fmt) => Some(fmt),
            Err(e) => {
                output.error(&e.to_string());
                std::process::exit(1);
            }
        }
    } else if args.len() > 2 && args[2].starts_with("--") {
        // Handle --format flag
        None
    } else {
        None
    };

    commands::recover::execute_with_output(session_id, target_format, config, output.clone()).await
}

fn print_usage() {
    println!("============================================================");
    println!("Audio Recorder Manager - Rust Edition");
    println!("============================================================");
    println!();
    println!("Usage:");
    println!("  audio-recorder-manager record <duration> [format] [quality]");
    println!("  audio-recorder-manager stop [session_id]");
    println!("  audio-recorder-manager recover [session_id] [format]");
    println!("  audio-recorder-manager status");
    println!();
    println!("Examples:");
    println!("  audio-recorder-manager record 30 wav           # Record for 30 seconds");
    println!("  audio-recorder-manager record -1 m4a standard  # Manual mode");
    println!("  audio-recorder-manager stop                    # Stop latest recording");
    println!("  audio-recorder-manager stop rec-20250109_120000 # Stop specific session");
    println!("  audio-recorder-manager recover                 # Recover all incomplete recordings");
    println!("  audio-recorder-manager recover rec-20250109_120000 # Recover specific session");
    println!("  audio-recorder-manager recover rec-20250109_120000 m4a # Recover and convert to M4A");
    println!("  audio-recorder-manager status                  # Show system audio devices");
    println!();
}
