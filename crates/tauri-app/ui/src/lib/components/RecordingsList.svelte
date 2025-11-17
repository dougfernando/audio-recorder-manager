<script>
  import { invoke } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { recordings, formatFileSize } from '../stores';
  import TranscriptViewer from './TranscriptViewer.svelte';

  let isLoading = false;
  let isDeleting = {};
  let isTranscribing = {};
  let transcriptPath = {};
  let transcriptionProgress = {}; // { filename: { step, progress, message } }
  let viewingTranscript = null; // { path: string, name: string }
  let progressPollingIntervals = {};

  // Check for transcripts whenever recordings change
  $: if ($recordings.length > 0) {
    checkForTranscripts();
  }

  async function loadRecordings() {
    isLoading = true;
    try {
      const result = await invoke('list_recordings');
      recordings.set(result);
      // Check for transcripts after recordings are loaded
      await checkForTranscripts();
    } catch (error) {
      console.error('Failed to load recordings:', error);
    } finally {
      isLoading = false;
    }
  }

  async function openRecording(path) {
    console.log('Opening recording:', path);
    try {
      const result = await invoke('open_recording', { filePath: path });
      console.log('Successfully opened:', result);
    } catch (error) {
      console.error('Failed to open recording:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      alert(`Failed to open recording: ${errorMsg}\n\nPath: ${path}`);
    }
  }

  async function deleteRecording(recording) {
    console.log('Delete requested for:', recording.filename);

    try {
      // Use Tauri's native dialog API for confirmation
      const confirmed = await ask(
        `Are you sure you want to delete "${recording.filename}"?`,
        {
          title: 'Delete Recording',
          type: 'warning'
        }
      );
      console.log('Confirmation result:', confirmed);

      if (!confirmed) {
        console.log('Deletion cancelled by user');
        return;
      }

      isDeleting[recording.filename] = true;
      console.log('Starting deletion of:', recording.path);

      const result = await invoke('delete_recording', {
        filePath: recording.path
      });
      console.log('Delete result:', result);
      console.log('Successfully deleted:', recording.filename);
      await loadRecordings();
    } catch (error) {
      console.error('Failed to delete recording:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      alert(`Failed to delete recording: ${errorMsg}`);
    } finally {
      delete isDeleting[recording.filename];
      isDeleting = isDeleting; // Trigger reactivity
    }
  }

  async function transcribeRecording(recording) {
    console.log('Transcribe requested for:', recording.filename);

    // Generate session ID for progress tracking
    const sessionId = `transcribe_${Date.now()}`;

    try {
      isTranscribing[recording.filename] = true;
      isTranscribing = isTranscribing; // Trigger reactivity

      // Start polling for progress
      startProgressPolling(recording.filename, sessionId);

      const result = await invoke('transcribe_recording', {
        filePath: recording.path,
        sessionId: sessionId,
      });

      console.log('Transcription result:', result);

      if (result.success && result.transcript_file) {
        transcriptPath[recording.filename] = result.transcript_file;
        transcriptPath = transcriptPath; // Trigger reactivity
        alert(`Transcription completed!\n\nSaved to: ${result.transcript_file}`);
      } else {
        throw new Error(result.error || 'Transcription failed');
      }
    } catch (error) {
      console.error('Failed to transcribe recording:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      alert(`Failed to transcribe recording: ${errorMsg}`);
    } finally {
      // Stop polling
      stopProgressPolling(recording.filename);
      delete isTranscribing[recording.filename];
      isTranscribing = isTranscribing; // Trigger reactivity
    }
  }

  function viewTranscript(recording) {
    const mdPath = transcriptPath[recording.filename] || recording.path.replace(/\.(wav|m4a)$/i, '.md');
    viewingTranscript = {
      path: mdPath,
      name: recording.filename
    };
  }

  function closeTranscriptViewer() {
    viewingTranscript = null;
  }

  async function checkForTranscripts() {
    // Check if .md files exist for each recording
    for (const recording of $recordings) {
      try {
        const exists = await invoke('check_transcript_exists', {
          filePath: recording.path
        });
        if (exists) {
          // Get the actual transcript path from backend
          const mdPath = await invoke('get_transcript_path', {
            filePath: recording.path
          });
          transcriptPath[recording.filename] = mdPath;
        }
      } catch (error) {
        console.error('Failed to check transcript for', recording.filename, error);
      }
    }
    transcriptPath = transcriptPath; // Trigger reactivity
  }

  function startProgressPolling(filename, sessionId) {
    // Clear any existing interval
    if (progressPollingIntervals[filename]) {
      clearInterval(progressPollingIntervals[filename]);
    }

    // Poll for progress every 500ms
    progressPollingIntervals[filename] = setInterval(async () => {
      try {
        const status = await invoke('get_transcription_status', {
          sessionId: sessionId
        });

        if (status) {
          transcriptionProgress[filename] = {
            step: status.step,
            progress: status.progress,
            message: status.message
          };
          transcriptionProgress = transcriptionProgress; // Trigger reactivity
        }
      } catch (error) {
        console.error('Failed to get transcription status:', error);
      }
    }, 500);
  }

  function stopProgressPolling(filename) {
    if (progressPollingIntervals[filename]) {
      clearInterval(progressPollingIntervals[filename]);
      delete progressPollingIntervals[filename];
      delete transcriptionProgress[filename];
      transcriptionProgress = transcriptionProgress; // Trigger reactivity
    }
  }
</script>

<div class="recordings-container">
  <div class="header">
    <h2>Recordings</h2>
    <button class="btn btn-secondary" on:click={loadRecordings} disabled={isLoading}>
      {#if isLoading}
        <svg class="spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
        Loading...
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
        </svg>
        Refresh
      {/if}
    </button>
  </div>

  {#if $recordings && $recordings.length > 0}
    <div class="recordings-grid">
      {#each $recordings as recording}
        <div class="recording-card card">
          <div class="recording-header">
            <div class="recording-icon {recording.format === 'wav' ? 'wav' : 'm4a'}">
              {#if recording.format === 'wav'}
                <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
                </svg>
              {:else}
                <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
                  <path d="M20 6v2h-2V6h2z" opacity="0.5"/>
                </svg>
              {/if}
            </div>
            <div class="format-badge {recording.format}">{recording.format.toUpperCase()}</div>
          </div>

          <div class="recording-info">
            <div class="recording-name" title={recording.filename}>
              {recording.filename}
            </div>
            <div class="recording-meta">
              <span>{formatFileSize(recording.size)}</span>
              <span>•</span>
              <span>{recording.created}</span>
            </div>
          </div>

          <div class="recording-actions">
            <button
              class="btn btn-primary btn-sm"
              on:click={() => openRecording(recording.path)}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="5 3 19 12 5 21 5 3"/>
              </svg>
              Play
            </button>
            <button
              class="btn btn-success btn-sm"
              on:click={() => transcribeRecording(recording)}
              disabled={isTranscribing[recording.filename]}
            >
              {#if isTranscribing[recording.filename]}
                <svg class="spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12a9 9 0 11-6.219-8.56"/>
                </svg>
                Transcribing...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                  <line x1="16" y1="13" x2="8" y2="13"/>
                  <line x1="16" y1="17" x2="8" y2="17"/>
                  <line x1="10" y1="9" x2="8" y2="9"/>
                </svg>
                Transcribe
              {/if}
            </button>
            <button
              class="btn btn-info btn-sm"
              on:click={() => viewTranscript(recording)}
              disabled={!transcriptPath[recording.filename]}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              View Transcript
            </button>
            <button
              class="btn btn-danger btn-sm"
              on:click={() => deleteRecording(recording)}
              disabled={isDeleting[recording.filename]}
            >
              {#if isDeleting[recording.filename]}
                Deleting...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                </svg>
                Delete
              {/if}
            </button>
          </div>

          {#if transcriptionProgress[recording.filename]}
            <div class="transcription-progress">
              <div class="progress-header">
                <span class="progress-step">{transcriptionProgress[recording.filename].step}</span>
                <span class="progress-percentage">{transcriptionProgress[recording.filename].progress}%</span>
              </div>
              <div class="progress-bar">
                <div class="progress-fill" style="width: {transcriptionProgress[recording.filename].progress}%"></div>
              </div>
              <div class="progress-message">{transcriptionProgress[recording.filename].message}</div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {:else if $recordings.length === 0 && isLoading}
    <!-- Skeleton loading UI -->
    <div class="recordings-grid">
      {#each Array(3) as _, i}
        <div class="recording-card card skeleton">
          <div class="skeleton-header">
            <div class="skeleton-icon"></div>
            <div class="skeleton-badge"></div>
          </div>
          <div class="skeleton-info">
            <div class="skeleton-title"></div>
            <div class="skeleton-meta"></div>
          </div>
          <div class="skeleton-actions">
            <div class="skeleton-btn"></div>
            <div class="skeleton-btn"></div>
          </div>
        </div>
      {/each}
    </div>
  {:else if $recordings.length === 0}
    <!-- Empty state when no recordings and not loading -->
    <div class="empty-state">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3"/>
        <circle cx="18" cy="16" r="3"/>
      </svg>
      <p>No recordings found</p>
      <small>Start a recording to see it appear here</small>
    </div>
  {/if}
</div>

{#if viewingTranscript}
  <TranscriptViewer
    transcriptPath={viewingTranscript.path}
    recordingName={viewingTranscript.name}
    onClose={closeTranscriptViewer}
  />
{/if}

<style>
  .recordings-container {
    max-width: 1200px;
    margin: 0 auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xxl);
  }

  h2 {
    font-size: 24px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .recordings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: var(--spacing-xl);
  }

  .recording-card {
    display: flex;
    flex-direction: column;
    padding: var(--spacing-lg);
    transition: all 0.2s ease;
  }

  .recording-card:hover {
    box-shadow: var(--elevation-flyout);
    transform: translateY(-2px);
  }

  .recording-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  .recording-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: var(--corner-radius-large);
    color: white;
  }

  .recording-icon.wav {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .recording-icon.m4a {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  }

  .format-badge {
    padding: 4px var(--spacing-sm);
    border-radius: 10px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.5px;
  }

  .format-badge.wav {
    background-color: var(--info-bg);
    color: var(--info);
  }

  .format-badge.m4a {
    background-color: var(--success-bg);
    color: var(--success);
  }

  .recording-info {
    flex: 1;
    margin-bottom: var(--spacing-lg);
  }

  .recording-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recording-meta {
    font-size: 12px;
    color: var(--text-secondary);
    display: flex;
    gap: 6px;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .recording-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-sm);
  }

  .btn-sm {
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: 13px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxxl) var(--spacing-lg);
    color: var(--text-tertiary);
  }

  .empty-state svg {
    margin-bottom: var(--spacing-lg);
    opacity: 0.3;
  }

  .empty-state p {
    font-size: 18px;
    font-weight: 500;
    margin-bottom: 6px;
  }

  .empty-state small {
    font-size: 14px;
  }

  .transcription-progress {
    margin-top: var(--spacing-md);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--stroke-surface);
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .progress-step {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent-default);
    text-transform: capitalize;
  }

  .progress-percentage {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    background-color: rgba(0, 103, 192, 0.1);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: var(--spacing-xs);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-default) 0%, var(--accent-secondary) 100%);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .progress-message {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  /* Skeleton Loading Styles */
  .recording-card.skeleton {
    pointer-events: none;
    user-select: none;
  }

  @keyframes skeleton-pulse {
    0%, 100% {
      opacity: 0.6;
    }
    50% {
      opacity: 0.3;
    }
  }

  .skeleton-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  .skeleton-icon {
    width: 48px;
    height: 48px;
    background: linear-gradient(90deg, rgba(0, 103, 192, 0.08) 0%, rgba(0, 103, 192, 0.12) 100%);
    border-radius: 50%;
    animation: skeleton-pulse 1.5s ease-in-out infinite;
  }

  .skeleton-badge {
    width: 50px;
    height: 24px;
    background: linear-gradient(90deg, rgba(0, 103, 192, 0.08) 0%, rgba(0, 103, 192, 0.12) 100%);
    border-radius: var(--corner-radius-small);
    animation: skeleton-pulse 1.5s ease-in-out infinite 0.2s;
  }

  .skeleton-info {
    margin-bottom: var(--spacing-lg);
  }

  .skeleton-title {
    width: 80%;
    height: 18px;
    background: linear-gradient(90deg, rgba(0, 103, 192, 0.08) 0%, rgba(0, 103, 192, 0.12) 100%);
    border-radius: var(--corner-radius-small);
    margin-bottom: var(--spacing-sm);
    animation: skeleton-pulse 1.5s ease-in-out infinite 0.1s;
  }

  .skeleton-meta {
    width: 60%;
    height: 14px;
    background: linear-gradient(90deg, rgba(0, 103, 192, 0.08) 0%, rgba(0, 103, 192, 0.12) 100%);
    border-radius: var(--corner-radius-small);
    animation: skeleton-pulse 1.5s ease-in-out infinite 0.3s;
  }

  .skeleton-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .skeleton-btn {
    flex: 1;
    height: 36px;
    background: linear-gradient(90deg, rgba(0, 103, 192, 0.08) 0%, rgba(0, 103, 192, 0.12) 100%);
    border-radius: var(--corner-radius-medium);
    animation: skeleton-pulse 1.5s ease-in-out infinite 0.4s;
  }
</style>
