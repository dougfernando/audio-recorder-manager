//! Incremental audio chunk processing module
//!
//! During long recordings, instead of processing the entire audio at the end,
//! this module splits the recording into chunks (e.g., every 5 minutes) and
//! processes each chunk (merge + encode) in the background while recording continues.
//!
//! At the end of recording, only the last segment needs processing, followed by
//! a fast concatenation of all pre-processed chunks.

use anyhow::Result;
use std::path::PathBuf;
use tokio::task::JoinHandle;

use crate::domain::AudioFormat;
use crate::recorder::{merge_audio_streams_smart, RecordingQuality};

/// Default chunk duration in seconds (5 minutes)
pub const DEFAULT_CHUNK_DURATION_SECS: u64 = 300;

/// Represents a completed audio chunk ready for background processing
pub struct AudioChunk {
    pub chunk_index: u32,
    pub loopback_path: PathBuf,
    pub mic_path: PathBuf,
    pub output_path: PathBuf,
    pub loopback_has_audio: bool,
    pub mic_has_audio: bool,
}

/// Manages background processing of audio chunks during recording.
///
/// Chunks are submitted for processing as they complete, and their merge+encode
/// runs in parallel with ongoing recording. At the end, all chunks are
/// concatenated using FFmpeg's concat demuxer with `-c copy` (no re-encoding).
pub struct ChunkProcessor {
    quality: RecordingQuality,
    output_format: AudioFormat,
    processing_tasks: Vec<(u32, JoinHandle<Result<PathBuf>>)>,
    completed_chunks: Vec<(u32, PathBuf)>,
}

impl ChunkProcessor {
    pub fn new(quality: RecordingQuality, output_format: AudioFormat) -> Self {
        Self {
            quality,
            output_format,
            processing_tasks: Vec::new(),
            completed_chunks: Vec::new(),
        }
    }

    /// Submit a chunk for background processing (merge + encode).
    /// Processing runs in a tokio task and does not block the recording.
    pub fn submit_chunk(&mut self, chunk: AudioChunk) {
        let quality = self.quality.clone();
        let output_format = self.output_format;
        let chunk_index = chunk.chunk_index;

        tracing::info!(
            "Submitting chunk {} for background processing: {:?}",
            chunk_index,
            chunk.output_path.file_name().unwrap_or_default()
        );

        let handle = tokio::spawn(async move {
            merge_audio_streams_smart(
                &chunk.loopback_path,
                &chunk.mic_path,
                &chunk.output_path,
                chunk.loopback_has_audio,
                chunk.mic_has_audio,
                &quality,
                output_format,
                None, // No session_id for background chunks (skip progress UI)
                None, // No observer for background chunks
                0,
            )
            .await?;

            // Clean up temp WAV files for this chunk
            let _ = std::fs::remove_file(&chunk.loopback_path);
            let _ = std::fs::remove_file(&chunk.mic_path);

            tracing::info!(
                "Background chunk {} processing completed: {:?}",
                chunk_index,
                chunk.output_path.file_name().unwrap_or_default()
            );

            Ok(chunk.output_path)
        });

        self.processing_tasks.push((chunk_index, handle));
    }

    /// Wait for all background processing tasks to complete.
    pub async fn wait_all(&mut self) -> Result<()> {
        tracing::info!(
            "Waiting for {} background chunk(s) to complete...",
            self.processing_tasks.len()
        );

        for (index, task) in self.processing_tasks.drain(..) {
            match task.await {
                Ok(Ok(path)) => {
                    tracing::info!("Chunk {} ready: {:?}", index, path.file_name().unwrap_or_default());
                    self.completed_chunks.push((index, path));
                }
                Ok(Err(e)) => {
                    tracing::error!("Chunk {} processing failed: {}", index, e);
                    return Err(e);
                }
                Err(e) => {
                    anyhow::bail!("Chunk {} task panicked: {}", index, e);
                }
            }
        }

        // Sort by chunk index to maintain correct order
        self.completed_chunks.sort_by_key(|(idx, _)| *idx);

        tracing::info!(
            "All {} background chunks completed",
            self.completed_chunks.len()
        );
        Ok(())
    }

    /// Add a chunk that was processed synchronously (e.g., the last chunk).
    pub fn add_completed_chunk(&mut self, index: u32, path: PathBuf) {
        self.completed_chunks.push((index, path));
        self.completed_chunks.sort_by_key(|(idx, _)| *idx);
    }

    /// Returns the number of chunks (completed + in-progress).
    pub fn total_chunks(&self) -> usize {
        self.completed_chunks.len() + self.processing_tasks.len()
    }

    /// Concatenate all processed chunks into the final output file.
    /// Uses FFmpeg concat demuxer with `-c copy` for near-instant concatenation.
    pub async fn concatenate_chunks(&self, final_output: &PathBuf) -> Result<()> {
        let chunk_paths: Vec<&PathBuf> = self.completed_chunks.iter().map(|(_, p)| p).collect();

        if chunk_paths.is_empty() {
            anyhow::bail!("No chunks to concatenate");
        }

        if chunk_paths.len() == 1 {
            // Only one chunk - just rename/move it
            tracing::info!("Single chunk, moving directly to final output");
            std::fs::rename(chunk_paths[0], final_output)?;
            return Ok(());
        }

        tracing::info!(
            "Concatenating {} chunks into final output: {:?}",
            chunk_paths.len(),
            final_output.file_name().unwrap_or_default()
        );

        concatenate_audio_files(&chunk_paths, final_output).await?;

        // Clean up individual chunk files
        for path in &chunk_paths {
            let _ = std::fs::remove_file(path);
        }

        Ok(())
    }
}

/// Concatenate multiple audio files using FFmpeg concat demuxer.
/// Uses `-c copy` so there is no re-encoding - this is very fast.
async fn concatenate_audio_files(input_files: &[&PathBuf], output_path: &PathBuf) -> Result<()> {
    use tokio::process::Command;

    // Create temporary concat list file
    let list_path = output_path.with_extension("concat_list.txt");
    let mut list_content = String::new();
    for file in input_files {
        // Use forward slashes and escape single quotes for FFmpeg compatibility
        let path_str = file.to_string_lossy().replace('\\', "/");
        list_content.push_str(&format!("file '{}'\n", path_str.replace('\'', "'\\''")));
    }
    std::fs::write(&list_path, &list_content)?;

    tracing::info!(
        "FFmpeg concat: {} files -> {:?}",
        input_files.len(),
        output_path.file_name().unwrap_or_default()
    );

    let mut cmd = Command::new("ffmpeg");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output_path);

    let output = cmd.output().await?;

    // Cleanup list file
    let _ = std::fs::remove_file(&list_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg concatenation failed: {}", stderr);
    }

    tracing::info!("Concatenation completed successfully");
    Ok(())
}
