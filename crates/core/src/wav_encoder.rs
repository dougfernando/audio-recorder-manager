//! WAV streaming encoder
//!
//! This module provides a wrapper around the hound WAV writer that implements
//! the StreamingEncoder trait. This maintains backwards compatibility with
//! the existing WAV recording functionality while fitting into the new
//! encoder abstraction.

use anyhow::{Context, Result};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::streaming_encoder::StreamingEncoder;

/// WAV encoder that writes uncompressed PCM audio
pub struct WavStreamingEncoder {
    /// The hound WAV writer
    writer: WavWriter<BufWriter<File>>,
    /// WAV specification
    spec: WavSpec,
    /// Output file path (for logging)
    output_path: PathBuf,
}

impl WavStreamingEncoder {
    /// Create a new WAV streaming encoder
    ///
    /// # Arguments
    /// * `output_path` - Path to the output WAV file
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000)
    ///
    /// # Returns
    /// * `Ok(WavStreamingEncoder)` - The created encoder
    /// * `Err` - If WAV file creation failed
    pub fn new(output_path: PathBuf, channels: u32, sample_rate: u32) -> Result<Self> {
        tracing::info!(
            "Initializing WAV encoder: {}Hz, {} channels",
            sample_rate,
            channels
        );

        let spec = WavSpec {
            channels: channels as u16,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer =
            WavWriter::create(&output_path, spec).context("Failed to create WAV writer")?;

        tracing::info!("WAV encoder initialized: {:?}", output_path);

        Ok(Self {
            writer,
            spec,
            output_path,
        })
    }
}

impl StreamingEncoder for WavStreamingEncoder {
    fn write_samples(
        &mut self,
        samples: &[i16],
        _channels: u32,
        _sample_rate: u32,
    ) -> Result<()> {
        for &sample in samples {
            self.writer
                .write_sample(sample)
                .context("Failed to write WAV sample")?;
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<()> {
        tracing::info!("Finalizing WAV file: {:?}", self.output_path);

        self.writer
            .finalize()
            .context("Failed to finalize WAV file")?;

        // Check if output file was created
        if !self.output_path.exists() {
            anyhow::bail!(
                "WAV finalization completed but output file was not created: {:?}",
                self.output_path
            );
        }

        let file_size = std::fs::metadata(&self.output_path)?.len();
        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

        tracing::info!(
            "WAV encoding completed successfully: {:?} ({:.2} MB)",
            self.output_path,
            file_size_mb
        );

        Ok(())
    }

    fn estimated_size(&self) -> Option<u64> {
        // WAV writer doesn't expose the current file size easily,
        // so we return None here. The file size can be checked directly
        // on the filesystem if needed.
        None
    }

    fn is_healthy(&self) -> bool {
        // WAV writing is always healthy unless there's a disk error,
        // which would be caught during write_samples
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_wav_encoder_creation() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_wav_creation.wav");

        let encoder = WavStreamingEncoder::new(output_path.clone(), 2, 48000);
        assert!(encoder.is_ok());

        // Cleanup
        if output_path.exists() {
            let _ = std::fs::remove_file(&output_path);
        }
    }

    #[test]
    fn test_wav_encoder_write_and_finish() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_wav_write.wav");

        let mut encoder = WavStreamingEncoder::new(output_path.clone(), 2, 48000).unwrap();

        // Generate some test samples (1 second of 440Hz sine wave)
        let sample_rate = 48000;
        let frequency = 440.0;
        let mut samples = Vec::new();

        for i in 0..sample_rate * 2 {
            // 2 channels
            let t = (i / 2) as f64 / sample_rate as f64;
            let value = (2.0 * std::f64::consts::PI * frequency * t).sin();
            samples.push((value * 16384.0) as i16);
        }

        // Write samples
        let result = encoder.write_samples(&samples, 2, 48000);
        assert!(result.is_ok());

        // Finish encoding
        let result = Box::new(encoder).finish();
        assert!(result.is_ok());

        // Verify file was created and has content
        assert!(output_path.exists());
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 44); // WAV header is 44 bytes, should have more

        // Cleanup
        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn test_wav_encoder_health() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_wav_health.wav");

        let encoder = WavStreamingEncoder::new(output_path.clone(), 2, 48000).unwrap();

        assert!(encoder.is_healthy());

        // Cleanup
        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn test_wav_encoder_spec() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_wav_spec.wav");

        let encoder = WavStreamingEncoder::new(output_path.clone(), 1, 44100).unwrap();

        assert_eq!(encoder.spec.channels, 1);
        assert_eq!(encoder.spec.sample_rate, 44100);
        assert_eq!(encoder.spec.bits_per_sample, 16);

        // Cleanup
        let _ = std::fs::remove_file(output_path);
    }
}
