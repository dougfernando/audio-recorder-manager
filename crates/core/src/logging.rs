//! Logging utilities and configuration
//!
//! This module provides utilities for setting up dual-output logging (file + terminal),
//! log rotation, cleanup, and log level management.

use std::fs;
use std::path::{Path, PathBuf};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Default number of days to keep log files
const DEFAULT_LOG_RETENTION_DAYS: u64 = 7;

/// Gets the log directory for the application on Windows
///
/// Returns: `%APPDATA%\audio-recorder-manager\logs\`
pub fn get_log_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA")
        .unwrap_or_else(|_| {
            // Fallback to current directory if APPDATA is not set
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .to_string_lossy()
                .to_string()
        });

    let log_dir = PathBuf::from(app_data)
        .join("audio-recorder-manager")
        .join("logs");

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    log_dir
}

/// Cleans up old log files beyond the retention period
///
/// # Arguments
/// * `log_dir` - Directory containing log files
/// * `retention_days` - Number of days to keep logs (default: 7)
pub fn cleanup_old_logs(log_dir: &Path, retention_days: Option<u64>) {
    let retention_days = retention_days.unwrap_or(DEFAULT_LOG_RETENTION_DAYS);
    let cutoff_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (retention_days * 24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            if duration.as_secs() < cutoff_time {
                                // File is older than retention period
                                if let Err(e) = fs::remove_file(entry.path()) {
                                    tracing::warn!(
                                        file = ?entry.path(),
                                        error = %e,
                                        "Failed to remove old log file"
                                    );
                                } else {
                                    tracing::debug!(
                                        file = ?entry.path(),
                                        "Removed old log file"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Parse log level from string
pub fn parse_log_level(level: &str) -> Option<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Some(Level::TRACE),
        "debug" => Some(Level::DEBUG),
        "info" => Some(Level::INFO),
        "warn" | "warning" => Some(Level::WARN),
        "error" => Some(Level::ERROR),
        _ => None,
    }
}

/// Initialize dual-output logging for CLI applications
///
/// Sets up logging to both file (with rotation) and terminal (stderr)
///
/// # Arguments
/// * `log_level` - Optional log level override (defaults to RUST_LOG env var or "info")
/// * `log_to_file` - Whether to enable file logging (default: true)
/// * `log_to_terminal` - Whether to enable terminal logging (default: true)
pub fn init_cli_logging(
    log_level: Option<Level>,
    log_to_file: bool,
    log_to_terminal: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_dir();

    // Clean up old logs on startup
    cleanup_old_logs(&log_dir, Some(DEFAULT_LOG_RETENTION_DAYS));

    // Build the base filter
    let default_level = log_level.unwrap_or(Level::INFO);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(format!("audio_recorder_manager={}", default_level))
        });

    let mut layers = Vec::new();

    // Layer 1: File output with daily rotation
    if log_to_file {
        let file_appender = tracing_appender::rolling::daily(&log_dir, "cli.log");
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .boxed();
        layers.push(file_layer);
    }

    // Layer 2: Terminal output (stderr)
    if log_to_terminal {
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(false) // Cleaner terminal output
            .with_line_number(false)
            .with_file(false)
            .boxed();
        layers.push(stderr_layer);
    }

    // Combine all layers
    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init()?;

    // Bridge log crate to tracing (ignore error if already initialized)
    let _ = tracing_log::LogTracer::init();

    Ok(())
}

/// Initialize dual-output logging for Tauri applications
///
/// Sets up logging to both file (with rotation) and optionally terminal
///
/// # Arguments
/// * `log_level` - Optional log level override (defaults to RUST_LOG env var or "info")
/// * `enable_terminal` - Whether to enable terminal logging (default: debug builds only)
pub fn init_tauri_logging(
    log_level: Option<Level>,
    enable_terminal: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_dir();

    // Clean up old logs on startup (keep 30 days for Tauri app)
    cleanup_old_logs(&log_dir, Some(30));

    // Build the base filter
    let default_level = log_level.unwrap_or(Level::INFO);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(format!("audio_recorder_manager={}", default_level))
        });

    let mut layers = Vec::new();

    // Layer 1: File output with daily rotation (compact format)
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .with_file(true)
        .compact()
        .boxed();
    layers.push(file_layer);

    // Layer 2: Terminal output (only if enabled, typically for debug builds)
    if enable_terminal {
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(false)
            .with_line_number(false)
            .with_file(false)
            .boxed();
        layers.push(stderr_layer);
    }

    // Combine all layers
    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init()?;

    // Bridge log crate to tracing (ignore error if already initialized)
    let _ = tracing_log::LogTracer::init();

    Ok(())
}

/// Initialize test logging (for use in tests)
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("audio_recorder_manager=debug"))
        )
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_dir() {
        let log_dir = get_log_dir();
        assert!(log_dir.ends_with("audio-recorder-manager\\logs"));
    }

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("trace"), Some(Level::TRACE));
        assert_eq!(parse_log_level("debug"), Some(Level::DEBUG));
        assert_eq!(parse_log_level("info"), Some(Level::INFO));
        assert_eq!(parse_log_level("warn"), Some(Level::WARN));
        assert_eq!(parse_log_level("warning"), Some(Level::WARN));
        assert_eq!(parse_log_level("error"), Some(Level::ERROR));
        assert_eq!(parse_log_level("invalid"), None);
    }

    #[test]
    fn test_cleanup_old_logs() {
        // Create a temporary log directory
        let temp_dir = std::env::temp_dir().join("test_logs");
        let _ = fs::create_dir_all(&temp_dir);

        // Test cleanup doesn't crash
        cleanup_old_logs(&temp_dir, Some(7));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
