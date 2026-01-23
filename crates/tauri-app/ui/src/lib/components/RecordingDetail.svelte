<script>
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { ask } from "@tauri-apps/plugin-dialog";
    import { formatFileSize, formatTime } from "../stores";
    import TranscriptViewer from "./TranscriptViewer.svelte";
    import KeyboardShortcut from "./KeyboardShortcut.svelte";
    import { createEventDispatcher, onMount, onDestroy } from "svelte";

    export let recording;
    export let onBack;
    export let shouldOpenTranscriptViewer = false;

    const dispatch = createEventDispatcher();

    // Gemini file size limit (100MB)
    const GEMINI_FILE_SIZE_LIMIT = 100 * 1024 * 1024;

    let isDeleting = false;
    let isTranscribing = false;
    let transcriptionProgress = null;
    let transcriptPath = null;
    let viewingTranscript = null;
    let progressPollingInterval = null;
    let transcriptPreview = null;

    // Compression state
    let showCompressionDialog = false;
    let compressionEstimate = null;
    let isCompressing = false;
    let compressionProgress = null;
    let selectedCompressionQuality = null;

    // Audio player state
    let audioElement = null;
    let isPlaying = false;
    let currentTime = 0;
    let duration = 0;
    let volume = 1.0;
    let isSeeking = false;
    let audioSrc = "";
    let waveformData = [];
    let isLoadingWaveform = false;
    let waveformError = null;

    // Check for transcript on mount
    $: if (recording) {
        checkForTranscript();
        initializeAudioSrc();
        loadWaveform();
    }

    // Initialize audio source when recording changes
    function initializeAudioSrc() {
        if (recording && recording.path) {
            const newSrc = convertFileSrc(recording.path);
            console.log("Initializing audio source:", newSrc);
            console.log("Original path:", recording.path);
            audioSrc = newSrc;
            // Reset player state
            isPlaying = false;
            currentTime = 0;
            duration = 0;

            // Force reload of audio element
            if (audioElement) {
                audioElement.load();
            }
        }
    }

    // Load waveform data using ffmpeg
    async function loadWaveform() {
        if (!recording || !recording.path) return;

        isLoadingWaveform = true;
        waveformError = null;

        try {
            const data = await invoke("generate_waveform", {
                filePath: recording.path,
                samples: 100,
            });
            waveformData = data;
        } catch (error) {
            console.error("Failed to load waveform:", error);
            waveformError = error;
            // Fallback to empty waveform
            waveformData = Array(100).fill(0.3);
        } finally {
            isLoadingWaveform = false;
        }
    }

    // Audio player functions
    async function togglePlayPause() {
        if (!audioElement) return;

        try {
            if (isPlaying) {
                audioElement.pause();
            } else {
                await audioElement.play();
            }
        } catch (error) {
            console.error("Error playing audio:", error);
        }
    }

    function handlePlayPause() {
        isPlaying = !audioElement.paused;
    }

    function handleTimeUpdate() {
        if (!isSeeking && audioElement) {
            currentTime = audioElement.currentTime;
        }
    }

    function handleLoadedMetadata() {
        if (audioElement) {
            duration = audioElement.duration;
            console.log("Audio duration loaded:", duration);
        }
    }

    function handleEnded() {
        isPlaying = false;
        currentTime = 0;
    }

    function handleError(event) {
        console.error("Audio loading error:", event);
        console.error("Audio element error:", audioElement?.error);
    }

    function handleSeek(event) {
        if (!audioElement) return;
        const rect = event.currentTarget.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const percentage = x / rect.width;
        const newTime = percentage * duration;
        audioElement.currentTime = newTime;
        currentTime = newTime;
    }

    function handleSeekStart() {
        isSeeking = true;
    }

    function handleSeekEnd() {
        isSeeking = false;
    }

    function handleVolumeChange(event) {
        if (audioElement) {
            audioElement.volume = event.target.value;
            volume = event.target.value;
        }
    }

    // Cleanup on unmount
    onDestroy(() => {
        if (audioElement) {
            audioElement.pause();
            audioElement.src = "";
        }
    });

    async function deleteRecording() {
        try {
            const confirmed = await ask(
                `Are you sure you want to delete "${recording.filename}"? This action cannot be undone.`,
                { title: "Confirm Deletion", type: "warning" },
            );
            if (!confirmed) return;

            isDeleting = true;
            await invoke("delete_recording", { filePath: recording.path });

            // Notify parent that deletion was successful
            dispatch("deleted");

            // The parent will handle closing this view
        } catch (error) {
            console.error("Failed to delete recording:", error);
            alert(`Failed to delete recording: ${error}`);
        } finally {
            isDeleting = false;
        }
    }

    let isRenaming = false;
    let newName = "";
    let renameError = null;

    function getFileNameWithoutExtension(filename) {
        return filename.replace(/\.[^/.]+$/, "");
    }

    function startRename() {
        isRenaming = true;
        newName = getFileNameWithoutExtension(recording.filename);
        renameError = null;
    }

    function cancelRename() {
        isRenaming = false;
        renameError = null;
    }

    async function saveRename() {
        if (!newName || newName.trim() === "") {
            renameError = "Filename cannot be empty.";
            return;
        }

        if (newName === getFileNameWithoutExtension(recording.filename)) {
            cancelRename();
            return;
        }

        renameError = null;
        try {
            const updatedRecording = await invoke("rename_recording", {
                oldPath: recording.path,
                newFilename: newName.trim(),
            });

            dispatch("renamed", updatedRecording);
            isRenaming = false;
        } catch (error) {
            console.error("Failed to rename recording:", error);
            renameError = error;
        }
    }

    async function checkForTranscript() {
        if (!recording) return;

        try {
            // First, check if transcript exists
            const exists = await invoke("check_transcript_exists", {
                filePath: recording.path,
            });

            console.log("[RecordingDetail] Transcript exists:", exists);

            if (exists) {
                // Get the transcript path
                const result = await invoke("get_transcript_path", {
                    filePath: recording.path,
                });
                console.log(
                    "[RecordingDetail] Transcript path result:",
                    result,
                );
                transcriptPath = result;
                await loadTranscriptPreview();
            } else {
                // No transcript exists
                transcriptPath = null;
                transcriptPreview = null;
            }
        } catch (error) {
            console.log(
                "[RecordingDetail] Error checking for transcript:",
                error,
            );
            // No transcript exists yet
            transcriptPath = null;
            transcriptPreview = null;
        }
    }

    async function loadTranscriptPreview() {
        if (!transcriptPath) return;

        try {
            const content = await invoke("read_transcript", {
                filePath: transcriptPath,
            });
            // Get first 300 characters as preview
            transcriptPreview =
                content.substring(0, 300) + (content.length > 300 ? "..." : "");
        } catch (error) {
            console.error("Failed to load transcript preview:", error);
            transcriptPreview = null;
        }
    }

    async function openRecording() {
        try {
            await invoke("open_recording", { filePath: recording.path });
        } catch (error) {
            console.error("Failed to open recording:", error);
            alert(`Failed to open recording: ${error}`);
        }
    }

    async function openFolder() {
        try {
            await invoke("open_folder", { filePath: recording.path });
        } catch (error) {
            console.error("Failed to open folder:", error);
            alert(`Failed to open folder: ${error}`);
        }
    }

    async function transcribeRecording() {
        // Check if transcript already exists and ask for confirmation
        if (transcriptPath) {
            const confirmed = await ask(
                "A transcript already exists for this recording. Do you want to overwrite it?",
                {
                    title: "Overwrite Transcript",
                    type: "warning",
                },
            );

            if (!confirmed) {
                return;
            }
        }

        const sessionId = `transcribe_${Date.now()}`;

        try {
            isTranscribing = true;
            startProgressPolling(sessionId);

            const result = await invoke("transcribe_recording", {
                filePath: recording.path,
                sessionId: sessionId,
            });

            console.log("Transcription complete:", result);
            transcriptPath = result;
            await checkForTranscript();
            await loadTranscriptPreview();
        } catch (error) {
            console.error("Failed to transcribe:", error);
            alert(`Transcription failed: ${error}`);
        } finally {
            isTranscribing = false;
            stopProgressPolling();
        }
    }

    // Wrapper function that checks file size before transcription
    async function handleTranscribeClick() {
        // Check file size first
        if (recording.size >= GEMINI_FILE_SIZE_LIMIT) {
            // File is too large, show compression dialog
            try {
                compressionEstimate = await invoke("get_compression_estimate", {
                    filePath: recording.path,
                });
                console.log("Compression estimate:", compressionEstimate);

                if (compressionEstimate.needs_compression) {
                    showCompressionDialog = true;
                    return;
                }
            } catch (error) {
                console.error("Failed to get compression estimate:", error);
                alert(`Failed to check file size: ${error}`);
                return;
            }
        }

        // File is small enough, proceed with normal transcription
        await transcribeRecording();
    }

    // Transcribe a compressed file
    async function transcribeCompressedFile(compressedPath) {
        // Check if transcript already exists and ask for confirmation
        if (transcriptPath) {
            const confirmed = await ask(
                "A transcript already exists for this recording. Do you want to overwrite it?",
                {
                    title: "Overwrite Transcript",
                    type: "warning",
                },
            );

            if (!confirmed) {
                return;
            }
        }

        const sessionId = `transcribe_${Date.now()}`;

        try {
            isTranscribing = true;
            startProgressPolling(sessionId);

            const result = await invoke("transcribe_recording", {
                filePath: compressedPath,
                sessionId: sessionId,
            });

            console.log("Transcription complete:", result);
            transcriptPath = result;
            await checkForTranscript();
            await loadTranscriptPreview();
        } catch (error) {
            console.error("Failed to transcribe:", error);
            alert(`Transcription failed: ${error}`);
        } finally {
            isTranscribing = false;
            stopProgressPolling();
        }
    }

    async function compressAndTranscribe(quality) {
        showCompressionDialog = false;
        selectedCompressionQuality = quality;

        const sessionId = `compress_${Date.now()}`;
        const originalPath = recording.path;

        try {
            isCompressing = true;
            startCompressionProgressPolling(sessionId);

            // Compress the file
            const compressedPath = await invoke("compress_for_transcription", {
                filePath: recording.path,
                quality: quality,
                sessionId: sessionId,
            });

            console.log("Compression complete:", compressedPath);
            stopCompressionProgressPolling();
            isCompressing = false;

            // Replace original file with compressed file and update recording info
            const updatedRecording = await invoke("replace_with_compressed", {
                originalPath: originalPath,
                compressedPath: compressedPath,
            });

            console.log("Replaced original with compressed:", updatedRecording);

            // Update the recording object with the new compressed file info
            recording = updatedRecording;

            // Notify parent that recording was replaced (passes both old and new info for proper list update)
            dispatch("replaced", {
                oldPath: originalPath,
                newRecording: updatedRecording,
            });

            // Explicitly reinitialize audio source and waveform for the new file
            initializeAudioSrc();
            await loadWaveform();

            // Now transcribe the NEW compressed file (use recording.path which is now updated)
            await transcribeCompressedFile(recording.path);
        } catch (error) {
            console.error("Failed to compress:", error);
            alert(`Compression failed: ${error}`);
            isCompressing = false;
            stopCompressionProgressPolling();
        }
    }

    function startCompressionProgressPolling(sessionId) {
        progressPollingInterval = setInterval(async () => {
            try {
                const progress = await invoke("get_transcription_progress", {
                    sessionId,
                });
                if (progress && progress.status === "compressing") {
                    compressionProgress = progress;
                }
            } catch (error) {
                console.error("Failed to get compression progress:", error);
            }
        }, 500);
    }

    function stopCompressionProgressPolling() {
        if (progressPollingInterval) {
            clearInterval(progressPollingInterval);
            progressPollingInterval = null;
        }
        compressionProgress = null;
    }

    function cancelCompressionDialog() {
        showCompressionDialog = false;
        compressionEstimate = null;
    }

    function startProgressPolling(sessionId) {
        progressPollingInterval = setInterval(async () => {
            try {
                const progress = await invoke("get_transcription_progress", {
                    sessionId,
                });
                if (progress) {
                    transcriptionProgress = progress;
                }
            } catch (error) {
                console.error("Failed to get progress:", error);
            }
        }, 500);
    }

    function stopProgressPolling() {
        if (progressPollingInterval) {
            clearInterval(progressPollingInterval);
            progressPollingInterval = null;
        }
        transcriptionProgress = null;
    }

    function viewTranscript() {
        if (transcriptPath) {
            viewingTranscript = {
                path: transcriptPath,
                name: recording.filename,
                recordingPath: recording.path,
            };
        }
    }

    function closeTranscriptViewer() {
        viewingTranscript = null;
    }

    // Watch for shouldOpenTranscriptViewer flag and open viewer automatically
    $: if (shouldOpenTranscriptViewer && transcriptPath && !viewingTranscript) {
        console.log(
            "[RecordingDetail] Auto-opening transcript viewer via keyboard shortcut",
        );
        viewTranscript();
        shouldOpenTranscriptViewer = false; // Reset flag
    }

    async function handleTranscribed() {
        // Reload transcript after re-transcription
        await checkForTranscript();
        await loadTranscriptPreview();
    }

    async function openInEditor() {
        if (transcriptPath) {
            try {
                await invoke("open_recording", { filePath: transcriptPath });
            } catch (error) {
                console.error("Failed to open transcript in editor:", error);
                alert(`Failed to open transcript: ${error}`);
            }
        }
    }
</script>

<div class="recording-detail">
    <!-- Header -->
    <div class="detail-header">
        <button class="back-btn" on:click={onBack}>
            <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
                <path
                    d="M11 2L5 8l6 6"
                    stroke="currentColor"
                    stroke-width="2"
                    fill="none"
                />
            </svg>
            Back
        </button>
    </div>

    <!-- Title -->
    <div class="title-container">
        {#if isRenaming}
            <div class="rename-form">
                <input
                    type="text"
                    bind:value={newName}
                    class="rename-input"
                    on:keydown={(e) => {
                        if (e.key === "Enter") saveRename();
                        if (e.key === "Escape") cancelRename();
                    }}
                    aria-label="New recording name"
                />
                <button class="btn btn-primary btn-sm" on:click={saveRename}
                    >Save</button
                >
                <button class="btn btn-secondary btn-sm" on:click={cancelRename}
                    >Cancel</button
                >
            </div>
            {#if renameError}
                <div class="rename-error">{renameError}</div>
            {/if}
        {:else}
            <h1 class="recording-title">{recording.filename}</h1>
        {/if}
    </div>

    <div class="recording-meta">
        {new Date(recording.created).toLocaleString()}
    </div>

    <!-- Audio Player Section -->
    <div class="player-section card">
        <!-- Hidden audio element -->
        <audio
            bind:this={audioElement}
            src={audioSrc}
            on:play={handlePlayPause}
            on:pause={handlePlayPause}
            on:timeupdate={handleTimeUpdate}
            on:loadedmetadata={handleLoadedMetadata}
            on:ended={handleEnded}
            on:error={handleError}
            preload="metadata"
        ></audio>

        <!-- Waveform and seek bar -->
        <div class="waveform-container">
            <div class="waveform-placeholder">
                {#if isLoadingWaveform}
                    <div class="waveform-loading">
                        <div class="spinner-small"></div>
                        <span>Analyzing audio...</span>
                    </div>
                {:else}
                    <svg
                        width="100%"
                        height="60"
                        viewBox="0 0 800 60"
                        preserveAspectRatio="none"
                    >
                        {#each waveformData as barHeight, i}
                            {@const progress =
                                duration > 0 ? currentTime / duration : 0}
                            {@const isPast =
                                i / waveformData.length <= progress}
                            {@const height = Math.max(2, barHeight * 50)}
                            <rect
                                x={i * 8}
                                y={30 - height / 2}
                                width="6"
                                {height}
                                fill={isPast
                                    ? "var(--accent-default)"
                                    : "var(--text-tertiary)"}
                                opacity={isPast ? "0.8" : "0.3"}
                            />
                        {/each}
                    </svg>
                {/if}
            </div>

            <!-- Seek bar overlay -->
            {#if !isLoadingWaveform}
                <div
                    class="seek-overlay"
                    on:click={handleSeek}
                    on:mousedown={handleSeekStart}
                    on:mouseup={handleSeekEnd}
                    on:keydown={(e) => {
                        if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
                            e.preventDefault();
                            const delta = e.key === "ArrowLeft" ? -5 : 5;
                            if (audioElement) {
                                audioElement.currentTime = Math.max(
                                    0,
                                    Math.min(duration, currentTime + delta),
                                );
                            }
                        }
                    }}
                    role="slider"
                    tabindex="0"
                    aria-label="Audio seek bar"
                    aria-valuemin="0"
                    aria-valuemax={duration}
                    aria-valuenow={currentTime}
                ></div>
            {/if}
        </div>

        <!-- Player controls -->
        <div class="player-controls">
            <div class="control-row">
                <button
                    class="btn-play"
                    on:click={togglePlayPause}
                    aria-label={isPlaying ? "Pause" : "Play"}
                >
                    {#if isPlaying}
                        <svg
                            width="20"
                            height="20"
                            viewBox="0 0 16 16"
                            fill="currentColor"
                        >
                            <path d="M3 2h3v12H3V2zm7 0h3v12h-3V2z" />
                        </svg>
                    {:else}
                        <svg
                            width="20"
                            height="20"
                            viewBox="0 0 16 16"
                            fill="currentColor"
                        >
                            <path d="M4 2l10 6-10 6V2z" />
                        </svg>
                    {/if}
                </button>

                <span class="time-display"
                    >{formatTime(Math.floor(currentTime))}</span
                >
                <span class="time-separator">/</span>
                <span class="time-display"
                    >{formatTime(Math.floor(duration || 0))}</span
                >

                <div class="volume-control">
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 16 16"
                        fill="currentColor"
                    >
                        <path d="M8 3L4 7H1v2h3l4 4V3zm3 1v8a4 4 0 000-8z" />
                    </svg>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        value={volume}
                        on:input={handleVolumeChange}
                        class="volume-slider"
                        aria-label="Volume"
                    />
                </div>
            </div>
        </div>
    </div>

    <!-- Metadata Section -->
    <div class="metadata-section">
        <h3 class="section-title">METADATA</h3>
        <div class="metadata-grid">
            <div class="metadata-item">
                <div class="metadata-label">Format</div>
                <div class="metadata-value">
                    {recording.format.toUpperCase()}
                </div>
            </div>
            <div class="metadata-item">
                <div class="metadata-label">Size</div>
                <div class="metadata-value">
                    {formatFileSize(recording.size)}
                </div>
            </div>
            <div class="metadata-item">
                <div class="metadata-label">Date & Time</div>
                <div class="metadata-value">
                    {new Date(recording.created).toLocaleString()}
                </div>
            </div>
        </div>
    </div>

    <!-- Transcript Section -->
    <div class="transcript-section">
        <h3 class="section-title">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M2 3h12v2H2V3zm0 4h12v2H2V7zm0 4h8v2H2v-2z" />
            </svg>
            Transcript
        </h3>

        {#if isTranscribing}
            <!-- Transcription in progress -->
            <div class="transcribing-state card">
                <div class="spinner"></div>
                <p>Transcribing...</p>
                {#if transcriptionProgress}
                    <div class="progress-info">
                        <p class="progress-step">
                            {transcriptionProgress.step}
                        </p>
                        {#if transcriptionProgress.message}
                            <p class="progress-message">
                                {transcriptionProgress.message}
                            </p>
                        {/if}
                    </div>
                {/if}
            </div>
        {:else if transcriptPath}
            <!-- Transcript exists - show preview and actions -->
            {#if transcriptPreview}
                <!-- Show preview card with all actions -->
                <div class="transcript-preview-card card">
                    <div class="transcript-preview-content">
                        <div class="transcript-icon">
                            <svg
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                            >
                                <path
                                    d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                                />
                                <polyline
                                    points="14 2 14 8 20 8"
                                    stroke="white"
                                    stroke-width="2"
                                    fill="none"
                                />
                                <line
                                    x1="16"
                                    y1="13"
                                    x2="8"
                                    y2="13"
                                    stroke="white"
                                    stroke-width="2"
                                />
                                <line
                                    x1="16"
                                    y1="17"
                                    x2="8"
                                    y2="17"
                                    stroke="white"
                                    stroke-width="2"
                                />
                                <line
                                    x1="10"
                                    y1="9"
                                    x2="8"
                                    y2="9"
                                    stroke="white"
                                    stroke-width="2"
                                />
                            </svg>
                        </div>
                        <div class="transcript-preview-text">
                            <div class="transcript-available-label">
                                <svg
                                    width="14"
                                    height="14"
                                    viewBox="0 0 16 16"
                                    fill="currentColor"
                                >
                                    <path
                                        d="M8 0a8 8 0 110 16A8 8 0 018 0zm3.707 6.707a1 1 0 00-1.414-1.414L7 8.586 5.707 7.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                    />
                                </svg>
                                Transcript Available
                            </div>
                            <p class="preview-text">{transcriptPreview}</p>
                        </div>
                    </div>
                    <div class="transcript-preview-actions">
                        <button
                            class="btn btn-primary"
                            on:click={viewTranscript}
                        >
                            <svg
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"
                                />
                                <circle cx="12" cy="12" r="3" />
                            </svg>
                            View Full Transcript
                            <KeyboardShortcut shortcut="VIEW_TRANSCRIPT" />
                        </button>
                        <button
                            class="btn btn-secondary"
                            on:click={openInEditor}
                        >
                            <svg
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                                />
                                <polyline points="15 3 21 3 21 9" />
                                <line x1="10" y1="14" x2="21" y2="3" />
                            </svg>
                            Open in Editor
                        </button>
                        <button
                            class="btn btn-secondary"
                            on:click={handleTranscribeClick}
                        >
                            <svg
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"
                                />
                            </svg>
                            Re-transcribe
                        </button>
                    </div>
                </div>
            {:else}
                <!-- Transcript exists but preview failed to load -->
                <div class="transcript-actions-row">
                    <button class="btn btn-primary" on:click={viewTranscript}>
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <path
                                d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"
                            />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                        View Transcript
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click={handleTranscribeClick}
                    >
                        <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <path
                                d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"
                            />
                        </svg>
                        Re-transcribe
                    </button>
                </div>
            {/if}
        {:else}
            <!-- No transcript - show generate button -->
            <button class="btn btn-primary" on:click={handleTranscribeClick}>
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 16 16"
                    fill="currentColor"
                >
                    <path d="M2 3h12v2H2V3zm0 4h12v2H2V7zm0 4h8v2H2v-2z" />
                </svg>
                Generate Transcript
            </button>
        {/if}
    </div>

    <!-- Actions -->
    <div class="actions-section">
        <button class="btn btn-secondary" on:click={startRename}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path
                    d="M12.7 4.7a1 1 0 0 0-1.4-1.4L2.5 12.1V13.5h1.4L12.7 4.7z M14.1 3.3l-1.4-1.4a1 1 0 0 0-1.4 0L10 3.3l2.8 2.8 1.3-1.3a1 1 0 0 0 0-1.4z"
                />
            </svg>
            Rename
        </button>
        <button class="btn btn-secondary" on:click={openFolder}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M2 3h5l2 2h5v8H2V3z" />
            </svg>
            Open Folder
        </button>
        <button class="btn btn-secondary" on:click={openRecording}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8 2a6 6 0 100 12A6 6 0 008 2zm-1 9V5l5 3-5 3z" />
            </svg>
            Play in Default App
        </button>
        <button
            class="btn btn-danger"
            on:click={deleteRecording}
            disabled={isDeleting}
        >
            {#if isDeleting}
                Deleting...
            {:else}
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <path
                        d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"
                    />
                </svg>
                Delete Recording
            {/if}
        </button>
    </div>
</div>

{#if viewingTranscript}
    <TranscriptViewer
        transcriptPath={viewingTranscript.path}
        recordingName={viewingTranscript.name}
        recordingPath={viewingTranscript.recordingPath}
        onClose={closeTranscriptViewer}
        onTranscribed={handleTranscribed}
    />
{/if}

<!-- Compression Dialog -->
{#if showCompressionDialog && compressionEstimate}
    <div
        class="modal-overlay"
        on:click={cancelCompressionDialog}
        on:keydown={(e) => e.key === "Escape" && cancelCompressionDialog()}
        role="dialog"
        aria-modal="true"
        tabindex="-1"
    >
        <div
            class="modal-content compression-dialog"
            on:click|stopPropagation
            role="document"
        >
            <div class="modal-header">
                <h2>File Too Large for Transcription</h2>
                <button
                    class="close-btn"
                    on:click={cancelCompressionDialog}
                    aria-label="Close"
                >
                    <svg
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <div class="modal-body">
                <div class="warning-banner">
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                    >
                        <path
                            d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"
                        />
                    </svg>
                    <div>
                        <strong>Gemini API Limit</strong>
                        <p>
                            This recording ({formatFileSize(
                                compressionEstimate.original_size,
                            )}) exceeds the 100MB file size limit for
                            transcription.
                        </p>
                    </div>
                </div>

                <div class="compression-options">
                    <h3>Compression Options</h3>
                    <p class="options-description">
                        You can compress the audio file to reduce its size.
                        {#if recording.format === "wav"}
                            The file will be converted from WAV to M4A format.
                        {/if}
                        Choose a quality level below:
                    </p>

                    {#if compressionEstimate.standard_quality?.would_fit}
                        <button
                            class="compression-option"
                            on:click={() => compressAndTranscribe("standard")}
                        >
                            <div class="option-header">
                                <span class="option-name">Standard Quality</span
                                >
                                <span class="option-badge recommended"
                                    >Recommended</span
                                >
                            </div>
                            <div class="option-details">
                                <span class="option-spec"
                                    >44.1kHz stereo, 128kbps</span
                                >
                                <span class="option-size"
                                    >Estimated: {formatFileSize(
                                        compressionEstimate.standard_quality
                                            .estimated_size,
                                    )}</span
                                >
                            </div>
                            <p class="option-description">
                                Good balance between quality and file size.
                                Suitable for most transcription needs.
                            </p>
                        </button>
                    {/if}

                    {#if compressionEstimate.quick_quality?.would_fit}
                        <button
                            class="compression-option"
                            on:click={() => compressAndTranscribe("quick")}
                        >
                            <div class="option-header">
                                <span class="option-name">Quick Quality</span>
                                {#if !compressionEstimate.standard_quality?.would_fit}
                                    <span class="option-badge">Only Option</span
                                    >
                                {/if}
                            </div>
                            <div class="option-details">
                                <span class="option-spec"
                                    >16kHz mono, 64kbps</span
                                >
                                <span class="option-size"
                                    >Estimated: {formatFileSize(
                                        compressionEstimate.quick_quality
                                            .estimated_size,
                                    )}</span
                                >
                            </div>
                            <p class="option-description">
                                Maximum compression for very long recordings.
                                Audio quality reduced but still suitable for
                                speech transcription.
                            </p>
                        </button>
                    {/if}

                    {#if !compressionEstimate.standard_quality?.would_fit && !compressionEstimate.quick_quality?.would_fit}
                        <div class="no-options-warning">
                            <svg
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                            >
                                <path
                                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
                                />
                            </svg>
                            <div>
                                <strong>Recording Too Long</strong>
                                <p>
                                    Even with maximum compression, this
                                    recording would exceed the 100MB limit.
                                    Consider splitting the recording into
                                    smaller segments for transcription.
                                </p>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            <div class="modal-footer">
                <button
                    class="btn btn-secondary"
                    on:click={cancelCompressionDialog}>Cancel</button
                >
            </div>
        </div>
    </div>
{/if}

<!-- Compression Progress -->
{#if isCompressing}
    <div class="modal-overlay" role="dialog" aria-modal="true" tabindex="-1">
        <div class="modal-content compression-progress-dialog" role="document">
            <div class="modal-header">
                <h2>Compressing Audio</h2>
            </div>

            <div class="modal-body">
                <div class="compression-progress-container">
                    <div class="spinner-large"></div>
                    <p class="progress-message">
                        {#if compressionProgress}
                            Compressing... {compressionProgress.compression_progress ||
                                0}%
                        {:else}
                            Starting compression...
                        {/if}
                    </p>
                    {#if compressionProgress?.estimated_remaining_secs}
                        <p class="progress-eta">
                            Estimated time remaining: {Math.floor(
                                compressionProgress.estimated_remaining_secs /
                                    60,
                            )}m {compressionProgress.estimated_remaining_secs %
                                60}s
                        </p>
                    {/if}
                    {#if compressionProgress?.compression_progress}
                        <div class="progress-bar-container">
                            <div
                                class="progress-bar"
                                style="width: {compressionProgress.compression_progress}%"
                            ></div>
                        </div>
                    {/if}
                </div>
                <p class="compression-info">
                    Compressing to {selectedCompressionQuality === "standard"
                        ? "Standard"
                        : "Quick"} quality for transcription...
                </p>
            </div>
        </div>
    </div>
{/if}

<style>
    .recording-detail {
        max-width: 900px;
        margin: 0 auto;
    }

    .detail-header {
        margin-bottom: var(--spacing-md);
    }

    .back-btn {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        padding: var(--spacing-sm) var(--spacing-md);
        background: transparent;
        color: var(--text-secondary);
        font-size: 14px;
        border-radius: var(--corner-radius-small);
        transition: all 0.08s ease;
    }

    .back-btn:hover {
        background: var(--card-background);
        color: var(--text-primary);
    }

    .recording-title {
        font-size: 24px;
        font-weight: 600;
        color: var(--text-primary);
        margin-bottom: var(--spacing-xs);
    }

    .recording-meta {
        font-size: 13px;
        color: var(--text-tertiary);
        margin-bottom: var(--spacing-lg);
    }

    .player-section {
        margin-bottom: var(--spacing-xxl);
    }

    .waveform-container {
        position: relative;
        margin-bottom: var(--spacing-md);
    }

    .waveform-placeholder {
        position: relative;
        background: var(--card-background-secondary);
        border-radius: var(--corner-radius-medium);
        padding: var(--spacing-md) var(--spacing-lg);
        overflow: hidden;
        min-height: 60px;
    }

    .waveform-loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 60px;
        gap: var(--spacing-sm);
        color: var(--text-tertiary);
        font-size: 13px;
    }

    .spinner-small {
        width: 20px;
        height: 20px;
        border: 2px solid var(--stroke-surface);
        border-top-color: var(--accent-default);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    .seek-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        cursor: pointer;
        z-index: 1;
    }

    .player-controls {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    .control-row {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
    }

    .btn-play {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background: var(--accent-default);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: var(--shadow-sm);
        transition: all 0.2s ease;
        flex-shrink: 0;
    }

    .btn-play:hover {
        background: var(--accent-secondary);
        box-shadow: var(--shadow-md);
        transform: scale(1.05);
    }

    .btn-play:active {
        transform: scale(0.95);
    }

    .time-display {
        font-size: 13px;
        color: var(--text-secondary);
        font-variant-numeric: tabular-nums;
        min-width: 45px;
    }

    .time-separator {
        color: var(--text-tertiary);
        margin: 0 var(--spacing-xs);
    }

    .volume-control {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        margin-left: auto;
        color: var(--text-tertiary);
    }

    .volume-slider {
        width: 80px;
        height: 4px;
        -webkit-appearance: none;
        appearance: none;
        background: var(--stroke-surface);
        border-radius: 2px;
        outline: none;
    }

    .volume-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--accent-default);
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .volume-slider::-webkit-slider-thumb:hover {
        background: var(--accent-secondary);
        transform: scale(1.2);
    }

    .volume-slider::-moz-range-thumb {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--accent-default);
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
    }

    .volume-slider::-moz-range-thumb:hover {
        background: var(--accent-secondary);
        transform: scale(1.2);
    }

    .metadata-section {
        margin-bottom: var(--spacing-lg);
    }

    .section-title {
        font-size: 12px;
        font-weight: 600;
        color: var(--text-tertiary);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: var(--spacing-md);
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
    }

    .metadata-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--spacing-sm);
    }

    .metadata-item {
        background: var(--card-background);
        padding: var(--spacing-sm);
        border-radius: var(--corner-radius-medium);
        border: 1px solid var(--stroke-surface);
    }

    .metadata-label {
        font-size: 11px;
        color: var(--text-tertiary);
        margin-bottom: var(--spacing-xs);
    }

    .metadata-value {
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .transcript-section {
        margin-bottom: var(--spacing-xxl);
    }

    .transcribing-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: var(--spacing-xxl);
        text-align: center;
    }

    .spinner {
        width: 32px;
        height: 32px;
        border: 3px solid var(--stroke-surface);
        border-top-color: var(--accent-default);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
        margin-bottom: var(--spacing-md);
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .progress-info {
        margin-top: var(--spacing-md);
        font-size: 13px;
    }

    .progress-step {
        color: var(--text-primary);
        font-weight: 500;
        margin-bottom: var(--spacing-xs);
    }

    .progress-message {
        color: var(--text-tertiary);
    }

    /* Transcript Preview Card */
    .transcript-preview-card {
        padding: var(--spacing-lg);
        background: linear-gradient(
            135deg,
            rgba(91, 157, 255, 0.1) 0%,
            rgba(91, 157, 255, 0.04) 100%
        );
        border: 1px solid rgba(91, 157, 255, 0.3);
    }

    .transcript-preview-content {
        display: flex;
        gap: var(--spacing-lg);
        margin-bottom: var(--spacing-lg);
        padding-bottom: var(--spacing-lg);
        border-bottom: 1px solid var(--stroke-surface);
    }

    .transcript-icon {
        flex-shrink: 0;
        width: 56px;
        height: 56px;
        border-radius: var(--corner-radius-medium);
        background: var(--gradient-primary);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        box-shadow: var(--shadow-sm);
    }

    .transcript-preview-text {
        flex: 1;
        min-width: 0;
    }

    .transcript-available-label {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        font-size: 13px;
        font-weight: 600;
        color: var(--success);
        margin-bottom: var(--spacing-sm);
    }

    .transcript-available-label svg {
        flex-shrink: 0;
    }

    .preview-text {
        font-size: 14px;
        line-height: 1.6;
        color: var(--text-secondary);
        margin: 0;
        overflow: hidden;
        display: -webkit-box;
        -webkit-line-clamp: 4;
        -webkit-box-orient: vertical;
    }

    .transcript-preview-actions {
        display: flex;
        gap: var(--spacing-sm);
        flex-wrap: wrap;
    }

    .transcript-actions-row {
        display: flex;
        gap: var(--spacing-sm);
        flex-wrap: wrap;
    }

    .actions-section {
        display: flex;
        gap: var(--spacing-md);
        flex-wrap: wrap;
    }

    .title-container {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        margin-bottom: var(--spacing-xs);
    }

    .rename-form {
        display: flex;
        gap: var(--spacing-sm);
        align-items: center;
        width: 100%;
    }

    .rename-input {
        flex-grow: 1;
        font-size: 20px;
        padding: var(--spacing-sm);
        border-radius: var(--corner-radius-small);
        border: 1px solid var(--stroke-surface);
        background: var(--card-background);
        color: var(--text-primary);
    }

    .rename-error {
        color: var(--danger);
        font-size: 13px;
        margin-top: var(--spacing-sm);
        width: 100%; /* Ensure it appears below the form */
    }

    @media (max-width: 768px) {
        .metadata-grid {
            grid-template-columns: 1fr;
        }

        .recording-title {
            font-size: 24px;
        }

        .actions-section {
            flex-direction: column;
        }

        .actions-section .btn {
            width: 100%;
        }

        .transcript-preview-content {
            flex-direction: column;
            gap: var(--spacing-md);
        }

        .transcript-icon {
            width: 48px;
            height: 48px;
        }

        .transcript-icon svg {
            width: 20px;
            height: 20px;
        }

        .transcript-preview-actions {
            flex-direction: column;
        }

        .transcript-preview-actions .btn {
            width: 100%;
        }

        .transcript-actions-row {
            flex-direction: column;
        }

        .transcript-actions-row .btn {
            width: 100%;
        }
    }

    /* Modal Overlay and Content */
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.6);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        backdrop-filter: blur(4px);
    }

    .modal-content {
        background: var(--card-background);
        border-radius: var(--corner-radius-large);
        border: 1px solid var(--stroke-surface);
        box-shadow: var(--shadow-lg);
        max-width: 500px;
        width: 90%;
        max-height: 90vh;
        overflow-y: auto;
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--spacing-lg);
        border-bottom: 1px solid var(--stroke-surface);
    }

    .modal-header h2 {
        font-size: 18px;
        font-weight: 600;
        color: var(--text-primary);
        margin: 0;
    }

    .close-btn {
        background: transparent;
        border: none;
        color: var(--text-tertiary);
        cursor: pointer;
        padding: var(--spacing-xs);
        border-radius: var(--corner-radius-small);
        transition: all 0.15s ease;
    }

    .close-btn:hover {
        background: var(--card-background-secondary);
        color: var(--text-primary);
    }

    .modal-body {
        padding: var(--spacing-lg);
    }

    .modal-footer {
        display: flex;
        justify-content: flex-end;
        gap: var(--spacing-sm);
        padding: var(--spacing-lg);
        border-top: 1px solid var(--stroke-surface);
    }

    /* Warning Banner */
    .warning-banner {
        display: flex;
        gap: var(--spacing-md);
        padding: var(--spacing-md);
        background: rgba(255, 171, 0, 0.1);
        border: 1px solid rgba(255, 171, 0, 0.3);
        border-radius: var(--corner-radius-medium);
        margin-bottom: var(--spacing-lg);
    }

    .warning-banner svg {
        flex-shrink: 0;
        color: var(--warning, #ffab00);
    }

    .warning-banner strong {
        display: block;
        color: var(--text-primary);
        margin-bottom: var(--spacing-xs);
    }

    .warning-banner p {
        margin: 0;
        font-size: 13px;
        color: var(--text-secondary);
    }

    /* Compression Options */
    .compression-options h3 {
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
        margin: 0 0 var(--spacing-xs) 0;
    }

    .options-description {
        font-size: 13px;
        color: var(--text-secondary);
        margin: 0 0 var(--spacing-md) 0;
    }

    .compression-option {
        display: block;
        width: 100%;
        text-align: left;
        padding: var(--spacing-md);
        background: var(--card-background-secondary);
        border: 1px solid var(--stroke-surface);
        border-radius: var(--corner-radius-medium);
        cursor: pointer;
        transition: all 0.15s ease;
        margin-bottom: var(--spacing-sm);
    }

    .compression-option:hover {
        background: var(--card-background);
        border-color: var(--accent-default);
        box-shadow: 0 0 0 2px rgba(91, 157, 255, 0.2);
    }

    .compression-option:last-child {
        margin-bottom: 0;
    }

    .option-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        margin-bottom: var(--spacing-xs);
    }

    .option-name {
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .option-badge {
        font-size: 11px;
        padding: 2px 8px;
        border-radius: 12px;
        background: var(--card-background);
        color: var(--text-secondary);
        border: 1px solid var(--stroke-surface);
    }

    .option-badge.recommended {
        background: rgba(91, 157, 255, 0.15);
        color: var(--accent-default);
        border-color: rgba(91, 157, 255, 0.3);
    }

    .option-details {
        display: flex;
        gap: var(--spacing-md);
        margin-bottom: var(--spacing-xs);
    }

    .option-spec {
        font-size: 12px;
        color: var(--text-tertiary);
    }

    .option-size {
        font-size: 12px;
        color: var(--accent-default);
        font-weight: 500;
    }

    .option-description {
        font-size: 12px;
        color: var(--text-secondary);
        margin: 0;
        line-height: 1.4;
    }

    /* No Options Warning */
    .no-options-warning {
        display: flex;
        gap: var(--spacing-md);
        padding: var(--spacing-md);
        background: rgba(255, 82, 82, 0.1);
        border: 1px solid rgba(255, 82, 82, 0.3);
        border-radius: var(--corner-radius-medium);
    }

    .no-options-warning svg {
        flex-shrink: 0;
        color: var(--danger, #ff5252);
    }

    .no-options-warning strong {
        display: block;
        color: var(--text-primary);
        margin-bottom: var(--spacing-xs);
    }

    .no-options-warning p {
        margin: 0;
        font-size: 13px;
        color: var(--text-secondary);
    }

    /* Compression Progress Dialog */
    .compression-progress-dialog {
        text-align: center;
    }

    .compression-progress-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: var(--spacing-lg) 0;
    }

    .spinner-large {
        width: 48px;
        height: 48px;
        border: 4px solid var(--stroke-surface);
        border-top-color: var(--accent-default);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
        margin-bottom: var(--spacing-md);
    }

    .compression-progress-container .progress-message {
        font-size: 16px;
        font-weight: 500;
        color: var(--text-primary);
        margin-bottom: var(--spacing-sm);
    }

    .progress-eta {
        font-size: 13px;
        color: var(--text-tertiary);
        margin: 0 0 var(--spacing-md) 0;
    }

    .progress-bar-container {
        width: 100%;
        height: 6px;
        background: var(--stroke-surface);
        border-radius: 3px;
        overflow: hidden;
    }

    .progress-bar {
        height: 100%;
        background: var(--accent-default);
        border-radius: 3px;
        transition: width 0.3s ease;
    }

    .compression-info {
        font-size: 13px;
        color: var(--text-secondary);
        margin: var(--spacing-md) 0 0 0;
    }
</style>
