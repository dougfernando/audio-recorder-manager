//! User-facing output abstraction
//!
//! This module provides a clean interface for user-facing messages.
//! Messages are displayed cleanly on the terminal AND logged to file for debugging.

use std::fmt;
use std::io::{self, Write};

/// Controls user-facing output (progress, warnings, errors)
/// This is separate from tracing/logging which is for developers
#[derive(Debug, Clone)]
pub struct UserOutput {
    /// If true, suppresses all output (except errors)
    quiet: bool,
    /// If true, use colored output
    use_color: bool,
}

impl UserOutput {
    /// Creates a new UserOutput with default settings
    pub fn new() -> Self {
        Self {
            quiet: false,
            use_color: Self::supports_color(),
        }
    }

    /// Creates a UserOutput that suppresses non-error messages
    pub fn quiet() -> Self {
        Self {
            quiet: true,
            use_color: false,
        }
    }

    /// Creates a UserOutput with explicit color setting
    pub fn with_color(use_color: bool) -> Self {
        Self {
            quiet: false,
            use_color,
        }
    }

    /// Checks if the terminal supports color
    fn supports_color() -> bool {
        // On Windows, check for TERM or NO_COLOR environment variables
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check if we're in a terminal that supports ANSI colors
        std::env::var("TERM").is_ok() ||
        std::env::var("WT_SESSION").is_ok() || // Windows Terminal
        std::env::var("ConEmuANSI").is_ok() // ConEmu
    }

    /// Prints an informational message to stdout
    pub fn info(&self, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::info!(user_output = true, "{}", msg_str);

        // Display to terminal
        if !self.quiet {
            println!("{}", msg_str);
        }
    }

    /// Prints a success message to stdout
    pub fn success(&self, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::info!(user_output = true, category = "success", "{}", msg_str);

        // Display to terminal
        if !self.quiet {
            if self.use_color {
                println!("\x1b[32m{}\x1b[0m", msg_str); // Green
            } else {
                println!("{}", msg_str);
            }
        }
    }

    /// Prints a warning message to stderr
    pub fn warning(&self, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::warn!(user_output = true, "{}", msg_str);

        // Display to terminal
        if !self.quiet {
            if self.use_color {
                eprintln!("\x1b[33m[Warning]\x1b[0m {}", msg_str); // Yellow
            } else {
                eprintln!("[Warning] {}", msg_str);
            }
        }
    }

    /// Prints an error message to stderr (always shown, even in quiet mode)
    pub fn error(&self, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::error!(user_output = true, "{}", msg_str);

        // Display to terminal
        if self.use_color {
            eprintln!("\x1b[31m[Error]\x1b[0m {}", msg_str); // Red
        } else {
            eprintln!("[Error] {}", msg_str);
        }
    }

    /// Prints a progress message with step numbers (e.g., "[1/4] Reading file...")
    pub fn progress(&self, step: usize, total: usize, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::info!(
            user_output = true,
            category = "progress",
            step = step,
            total = total,
            "{}",
            msg_str
        );

        // Display to terminal
        if !self.quiet {
            if self.use_color {
                println!("\x1b[36m[{}/{}]\x1b[0m {}", step, total, msg_str); // Cyan
            } else {
                println!("[{}/{}] {}", step, total, msg_str);
            }
        }
    }

    /// Prints a message with a custom prefix
    pub fn prefixed(&self, prefix: &str, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file
        tracing::info!(user_output = true, prefix = prefix, "{}", msg_str);

        // Display to terminal
        if !self.quiet {
            if self.use_color {
                println!("\x1b[36m[{}]\x1b[0m {}", prefix, msg_str); // Cyan
            } else {
                println!("[{}] {}", prefix, msg_str);
            }
        }
    }

    /// Prints a status update that overwrites the previous line (for progress bars, etc.)
    /// Note: Status updates are logged at debug level to avoid log noise
    pub fn status(&self, msg: impl fmt::Display) {
        let msg_str = msg.to_string();

        // Log to file at debug level (less noisy than info)
        tracing::debug!(user_output = true, category = "status", "{}", msg_str);

        // Display to terminal
        if !self.quiet {
            print!("\r{}", msg_str);
            let _ = io::stdout().flush();
        }
    }

    /// Clears the current status line
    pub fn clear_status(&self) {
        if !self.quiet {
            print!("\r\x1b[K"); // Clear line
            let _ = io::stdout().flush();
        }
    }
}

impl Default for UserOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_output_creation() {
        let output = UserOutput::new();
        assert!(!output.quiet);

        let quiet_output = UserOutput::quiet();
        assert!(quiet_output.quiet);

        let color_output = UserOutput::with_color(true);
        assert!(color_output.use_color);
    }

    #[test]
    fn test_quiet_mode() {
        let output = UserOutput::quiet();
        // These should not panic, just not print anything
        output.info("test");
        output.success("test");
        output.warning("test");
        // Error should still print
        output.error("test error");
    }
}
