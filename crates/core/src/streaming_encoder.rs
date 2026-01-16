//! Streaming audio encoder trait and types
//!
//! This module provides a trait for real-time audio encoding during recording.
//! Instead of writing to temporary WAV files and post-processing, encoders
//! accept PCM samples directly and encode them on-the-fly.

use anyhow::Result;
use std::path::PathBuf;

/// Trait for real-time audio encoding during recording
///
/// Implementations write PCM audio samples to a compressed format in real-time,
/// eliminating the need for post-processing and large temporary files.
pub trait StreamingEncoder: Send {
    /// Write PCM samples to the encoder
    ///
    /// # Arguments
    /// * `samples` - Interleaved PCM samples (i16 format, little-endian)
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000)
    ///
    /// # Returns
    /// * `Ok(())` if samples were written successfully
    /// * `Err` if encoding failed
    fn write_samples(&mut self, samples: &[i16], channels: u32, sample_rate: u32) -> Result<()>;

    /// Finish encoding and flush remaining data
    ///
    /// This should be called when recording is complete to ensure all data
    /// is written and the file is properly finalized.
    ///
    /// # Returns
    /// * `Ok(())` if finalization succeeded
    /// * `Err` if finalization failed
    fn finish(self: Box<Self>) -> Result<()>;

    /// Get estimated output file size in bytes (if available)
    ///
    /// This is useful for progress reporting and disk space monitoring.
    /// Returns None if size estimation is not supported.
    fn estimated_size(&self) -> Option<u64>;

    /// Check if encoder process is healthy
    ///
    /// For encoders that use external processes (like FFmpeg), this checks
    /// if the process is still running. For direct encoders, always returns true.
    ///
    /// # Returns
    /// * `true` if encoder is healthy and ready to accept more data
    /// * `false` if encoder has failed or is not responsive
    fn is_healthy(&self) -> bool;
}

/// Audio output format specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Uncompressed WAV (PCM 16-bit)
    /// - No compression
    /// - ~192 KB/sec for 48kHz stereo
    /// - Universal compatibility
    Wav,

    /// M4A (AAC encoding)
    /// - Good compression ratio
    /// - Specified bitrate in kbps
    /// - Excellent compatibility
    M4a { bitrate: u32 },

    /// MP3 (LAME encoding)
    /// - Good compression ratio
    /// - Specified bitrate in kbps
    /// - Universal compatibility
    Mp3 { bitrate: u32 },
}

impl OutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Wav => "wav",
            OutputFormat::M4a { .. } => "m4a",
            OutputFormat::Mp3 { .. } => "mp3",
        }
    }

    /// Get a human-readable name for this format
    pub fn name(&self) -> String {
        match self {
            OutputFormat::Wav => "WAV (PCM)".to_string(),
            OutputFormat::M4a { bitrate } => format!("M4A (AAC {}kbps)", bitrate),
            OutputFormat::Mp3 { bitrate } => format!("MP3 ({}kbps)", bitrate),
        }
    }

    /// Get the codec name for this format
    pub fn codec_name(&self) -> &str {
        match self {
            OutputFormat::Wav => "PCM",
            OutputFormat::M4a { .. } => "AAC",
            OutputFormat::Mp3 { .. } => "MP3",
        }
    }

    /// Check if this format requires FFmpeg
    pub fn requires_ffmpeg(&self) -> bool {
        matches!(self, OutputFormat::M4a { .. } | OutputFormat::Mp3 { .. })
    }

    /// Estimate file size per second for this format
    /// Returns bytes per second of audio
    pub fn bytes_per_second(&self, channels: u32, sample_rate: u32) -> u64 {
        match self {
            OutputFormat::Wav => {
                // PCM 16-bit: sample_rate * channels * 2 bytes per sample
                (sample_rate * channels * 2) as u64
            }
            OutputFormat::M4a { bitrate } | OutputFormat::Mp3 { bitrate } => {
                // Bitrate is in kbps, convert to bytes per second
                (*bitrate as u64 * 1000) / 8
            }
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        // Default to M4A 192kbps for good quality and size balance
        OutputFormat::M4a { bitrate: 192 }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Factory function to create an encoder based on output format
///
/// # Arguments
/// * `format` - The desired output format
/// * `path` - Output file path
/// * `channels` - Number of audio channels (1=mono, 2=stereo)
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// * `Ok(Box<dyn StreamingEncoder>)` - The created encoder
/// * `Err` - If encoder creation failed
pub fn create_encoder(
    format: &OutputFormat,
    path: PathBuf,
    channels: u32,
    sample_rate: u32,
) -> Result<Box<dyn StreamingEncoder>> {
    match format {
        OutputFormat::Wav => {
            let encoder = crate::wav_encoder::WavStreamingEncoder::new(path, channels, sample_rate)?;
            Ok(Box::new(encoder))
        }
        OutputFormat::M4a { bitrate } => {
            let encoder =
                crate::ffmpeg_pipe_encoder::FFmpegPipeEncoder::new_m4a(path, channels, sample_rate, *bitrate)?;
            Ok(Box::new(encoder))
        }
        OutputFormat::Mp3 { bitrate } => {
            let encoder =
                crate::ffmpeg_pipe_encoder::FFmpegPipeEncoder::new_mp3(path, channels, sample_rate, *bitrate)?;
            Ok(Box::new(encoder))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Wav.extension(), "wav");
        assert_eq!(OutputFormat::M4a { bitrate: 192 }.extension(), "m4a");
        assert_eq!(OutputFormat::Mp3 { bitrate: 192 }.extension(), "mp3");
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Wav.to_string(), "WAV (PCM)");
        assert_eq!(
            OutputFormat::M4a { bitrate: 192 }.to_string(),
            "M4A (AAC 192kbps)"
        );
        assert_eq!(
            OutputFormat::Mp3 { bitrate: 320 }.to_string(),
            "MP3 (320kbps)"
        );
    }

    #[test]
    fn test_output_format_bytes_per_second() {
        // WAV 48kHz stereo 16-bit = 48000 * 2 * 2 = 192000 bytes/sec
        assert_eq!(OutputFormat::Wav.bytes_per_second(2, 48000), 192000);

        // M4A 192kbps = 192 * 1000 / 8 = 24000 bytes/sec
        assert_eq!(
            OutputFormat::M4a { bitrate: 192 }.bytes_per_second(2, 48000),
            24000
        );

        // MP3 320kbps = 320 * 1000 / 8 = 40000 bytes/sec
        assert_eq!(
            OutputFormat::Mp3 { bitrate: 320 }.bytes_per_second(2, 48000),
            40000
        );
    }

    #[test]
    fn test_output_format_default() {
        let default = OutputFormat::default();
        assert!(matches!(default, OutputFormat::M4a { bitrate: 192 }));
    }

    #[test]
    fn test_output_format_requires_ffmpeg() {
        assert!(!OutputFormat::Wav.requires_ffmpeg());
        assert!(OutputFormat::M4a { bitrate: 192 }.requires_ffmpeg());
        assert!(OutputFormat::Mp3 { bitrate: 192 }.requires_ffmpeg());
    }
}
