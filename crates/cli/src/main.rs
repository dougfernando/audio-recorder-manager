use audio_recorder_manager_core::cli;
use audio_recorder_manager_core::logging;
use audio_recorder_manager_core::Result;
use std::env;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse log level from command line args if provided
    let args: Vec<String> = env::args().collect();
    let log_level = parse_log_level_from_args(&args);
    let (log_to_file, log_to_terminal) = parse_log_output_from_args(&args);

    // Initialize dual-output logging (file + terminal)
    if let Err(e) = logging::init_cli_logging(log_level, log_to_file, log_to_terminal) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    tracing::info!("Audio Recorder Manager CLI started");

    // Run the CLI
    let result = cli::run(args).await;

    if result.is_ok() {
        tracing::info!("CLI completed successfully");
    } else {
        tracing::error!("CLI completed with error");
    }

    result
}

/// Parse log level from command line arguments
fn parse_log_level_from_args(args: &[String]) -> Option<Level> {
    for (i, arg) in args.iter().enumerate() {
        if (arg == "--log-level" || arg == "-l") && i + 1 < args.len() {
            if let Some(level) = logging::parse_log_level(&args[i + 1]) {
                return Some(level);
            }
        }
    }
    None
}

/// Parse log output options from command line arguments
fn parse_log_output_from_args(args: &[String]) -> (bool, bool) {
    let mut log_to_file = true;
    let mut log_to_terminal = true;

    for arg in args {
        match arg.as_str() {
            "--no-log-file" => log_to_file = false,
            "--no-log-terminal" => log_to_terminal = false,
            _ => {}
        }
    }

    (log_to_file, log_to_terminal)
}
