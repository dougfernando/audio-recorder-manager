<script>
  import { invoke } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { recordings, formatFileSize } from '../stores';
  import TranscriptViewer from './TranscriptViewer.svelte';
  import RecordingDetail from './RecordingDetail.svelte';

  // Optional callback for when a recording is clicked (for parent navigation)
  export let onRecordingClick = null;

  let isLoading = false;
  let isDeleting = {};
  let isTranscribing = {};
  let transcriptPath = {};
  let transcriptionProgress = {}; // { filename: { step, progress, message } }
  let viewingTranscript = null; // { path: string, name: string }
  let progressPollingIntervals = {};
  let selectedRecording = null; // Recording object for detail view
  let renamingRecording = null; // filename of recording being renamed
  let newName = '';
  let renameError = null;

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

  function getFileNameWithoutExtension(filename) {
    return filename.replace(/\.[^/.]+$/, "");
  }

  function startRename(recording, event) {
    event.stopPropagation();
    renamingRecording = recording.filename;
    newName = getFileNameWithoutExtension(recording.filename);
    renameError = null;
  }

  function cancelRename() {
    renamingRecording = null;
    newName = '';
    renameError = null;
  }

  async function saveRename(recording, event) {
    event.stopPropagation();

    if (!newName || newName.trim() === '') {
      renameError = 'Filename cannot be empty.';
      return;
    }

    if (newName === getFileNameWithoutExtension(recording.filename)) {
      cancelRename();
      return;
    }

    renameError = null;
    try {
      await invoke('rename_recording', {
        oldPath: recording.path,
        newFilename: newName.trim(),
      });

      // Reload recordings to show updated name
      await loadRecordings();
      cancelRename();
    } catch (error) {
      console.error('Failed to rename recording:', error);
      renameError = error;
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
      name: recording.filename,
      recordingPath: recording.path
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

  function showRecordingDetail(recording) {
    console.log('Showing recording detail for:', recording);

    // If parent wants to handle navigation, use the callback
    if (onRecordingClick) {
      onRecordingClick(recording);
    } else {
      // Otherwise, use internal state (backward compatibility)
      selectedRecording = recording;
      console.log('selectedRecording set to:', selectedRecording);
    }
  }

  function closeRecordingDetail() {
    selectedRecording = null;
  }

  // Debug: log when selectedRecording changes
  $: console.log('[RecordingsList] selectedRecording changed:', selectedRecording);
</script>

{#if selectedRecording}
  <!-- Showing RecordingDetail -->
  <RecordingDetail
    recording={selectedRecording}
    onBack={closeRecordingDetail}
  />
{:else}
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
      <div class="recordings-list">
        {#each $recordings as recording (recording.path)}
          <div
            class="recording-item"
            on:click={() => showRecordingDetail(recording)}
            on:keydown={(e) => e.key === 'Enter' && showRecordingDetail(recording)}
            role="button"
            tabindex="0"
          >
            <div class="recording-content">
              <!-- Icon and Name -->
              <div class="recording-main">
                <div class="recording-icon {recording.format}">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
                  </svg>
                </div>
                <div class="recording-info">
                  {#if renamingRecording === recording.filename}
                    <!-- Rename input -->
                    <div class="rename-container" on:click|stopPropagation>
                      <input
                        type="text"
                        bind:value={newName}
                        class="rename-input"
                        on:keydown={(e) => {
                          if (e.key === 'Enter') saveRename(recording, e);
                          if (e.key === 'Escape') cancelRename();
                        }}
                        on:click|stopPropagation
                        aria-label="New recording name"
                        autofocus
                      />
                      <button class="btn-rename-save" on:click={(e) => saveRename(recording, e)} title="Save">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <polyline points="20 6 9 17 4 12"/>
                        </svg>
                      </button>
                      <button class="btn-rename-cancel" on:click|stopPropagation={cancelRename} title="Cancel">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <line x1="18" y1="6" x2="6" y2="18"/>
                          <line x1="6" y1="6" x2="18" y2="18"/>
                        </svg>
                      </button>
                    </div>
                    {#if renameError}
                      <div class="rename-error">{renameError}</div>
                    {/if}
                  {:else}
                    <div class="recording-name" title={recording.filename}>
                      {recording.filename}
                    </div>
                    <div class="recording-meta">
                      <span class="format-badge {recording.format}">{recording.format.toUpperCase()}</span>
                      <span class="meta-separator">•</span>
                      <span>{formatFileSize(recording.size)}</span>
                      <span class="meta-separator">•</span>
                      <span>{recording.created}</span>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Actions -->
              {#if renamingRecording !== recording.filename}
              <div class="recording-actions">
                <button
                  class="action-btn"
                  on:click|stopPropagation={() => openRecording(recording.path)}
                  title="Play in embedded player"
                  aria-label="Play recording"
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polygon points="5 3 19 12 5 21 5 3"/>
                  </svg>
                </button>

                {#if transcriptPath[recording.filename]}
                  <!-- View Transcript Button -->
                  <button
                    class="action-btn has-transcript"
                    on:click|stopPropagation={() => viewTranscript(recording)}
                    title="View transcript"
                    aria-label="View transcript"
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                      <circle cx="12" cy="12" r="3"/>
                    </svg>
                  </button>
                {:else}
                  <!-- Transcribe Button -->
                  <button
                    class="action-btn {isTranscribing[recording.filename] ? 'loading' : ''}"
                    on:click|stopPropagation={() => transcribeRecording(recording)}
                    disabled={isTranscribing[recording.filename]}
                    title="Transcribe recording"
                    aria-label="Transcribe recording"
                  >
                    {#if isTranscribing[recording.filename]}
                      <svg class="spin" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M21 12a9 9 0 11-6.219-8.56"/>
                      </svg>
                    {:else}
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                        <polyline points="14 2 14 8 20 8"/>
                        <line x1="16" y1="13" x2="8" y2="13"/>
                        <line x1="16" y1="17" x2="8" y2="17"/>
                        <line x1="10" y1="9" x2="8" y2="9"/>
                      </svg>
                    {/if}
                  </button>
                {/if}

                <button
                  class="action-btn"
                  on:click={(e) => startRename(recording, e)}
                  title="Rename recording"
                  aria-label="Rename recording"
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                  </svg>
                </button>

                <button
                  class="action-btn danger"
                  on:click|stopPropagation={() => deleteRecording(recording)}
                  disabled={isDeleting[recording.filename]}
                  title="Delete recording"
                  aria-label="Delete recording"
                >
                  {#if isDeleting[recording.filename]}
                    <svg class="spin" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M21 12a9 9 0 11-6.219-8.56"/>
                    </svg>
                  {:else}
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                    </svg>
                  {/if}
                </button>
              </div>
              {/if}
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
{/if}

{#if viewingTranscript}
  <TranscriptViewer
    transcriptPath={viewingTranscript.path}
    recordingName={viewingTranscript.name}
    recordingPath={viewingTranscript.recordingPath}
    onClose={closeTranscriptViewer}
    onTranscribed={async () => {
      await loadRecordings();
      await checkForTranscripts();
    }}
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

  .recordings-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--stroke-surface);
    border-radius: var(--corner-radius-medium);
    overflow: hidden;
  }

  .recording-item {
    background: var(--card-background);
    cursor: pointer;
    transition: all 0.15s ease;
    position: relative;
  }

  .recording-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--accent-default);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .recording-item:hover {
    background: var(--card-background-secondary);
  }

  .recording-item:hover::before {
    opacity: 1;
  }

  .recording-item:active {
    transform: scale(0.995);
  }

  .recording-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    gap: var(--spacing-lg);
  }

  .recording-main {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    flex: 1;
    min-width: 0;
  }

  .recording-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: var(--corner-radius-medium);
    color: white;
    flex-shrink: 0;
    transition: transform 0.2s ease;
  }

  .recording-item:hover .recording-icon {
    transform: scale(1.05);
  }

  .recording-icon.wav {
    background: linear-gradient(135deg, #0067C0 0%, #00A3E0 100%);
  }

  .recording-icon.m4a {
    background: linear-gradient(135deg, #FF3B30 0%, #FF6B6B 100%);
  }

  .recording-info {
    flex: 1;
    min-width: 0;
  }

  .recording-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recording-meta {
    font-size: 12px;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .format-badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .format-badge.wav {
    background: rgba(0, 103, 192, 0.1);
    color: #0067C0;
  }

  .format-badge.m4a {
    background: rgba(255, 59, 48, 0.1);
    color: #FF3B30;
  }

  .meta-separator {
    color: var(--stroke-surface);
  }

  .recording-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .action-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--corner-radius-small);
    background: transparent;
    border: none;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--accent-default);
    color: white;
    transform: scale(1.1);
  }

  .action-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .action-btn.has-transcript {
    color: var(--success);
  }

  .action-btn.has-transcript:hover:not(:disabled) {
    background: var(--success);
    color: white;
  }

  .action-btn.danger:hover:not(:disabled) {
    background: var(--danger);
    color: white;
  }

  .action-btn.loading {
    cursor: not-allowed;
  }

  .rename-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    width: 100%;
  }

  .rename-input {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
    padding: 4px 8px;
    border: 1px solid var(--accent-default);
    border-radius: var(--corner-radius-small);
    background: var(--card-background);
    color: var(--text-primary);
    outline: none;
  }

  .rename-input:focus {
    border-color: var(--accent-secondary);
    box-shadow: 0 0 0 2px rgba(0, 103, 192, 0.1);
  }

  .btn-rename-save,
  .btn-rename-cancel {
    width: 24px;
    height: 24px;
    border-radius: var(--corner-radius-small);
    background: transparent;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
    flex-shrink: 0;
  }

  .btn-rename-save {
    color: var(--success);
  }

  .btn-rename-save:hover {
    background: var(--success);
    color: white;
  }

  .btn-rename-cancel {
    color: var(--text-secondary);
  }

  .btn-rename-cancel:hover {
    background: var(--danger);
    color: white;
  }

  .rename-error {
    color: var(--danger);
    font-size: 11px;
    margin-top: 4px;
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
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--stroke-surface);
    background: var(--card-background-secondary);
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .progress-step {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-default);
    text-transform: capitalize;
  }

  .progress-percentage {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .progress-bar {
    width: 100%;
    height: 4px;
    background-color: rgba(0, 103, 192, 0.1);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: var(--spacing-xs);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-default) 0%, var(--accent-secondary) 100%);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .progress-message {
    font-size: 11px;
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

  /* Responsive Design */
  @media (max-width: 768px) {
    .recordings-container {
      padding: 0;
    }

    .header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-md);
      margin-bottom: var(--spacing-lg);
    }

    .header h2 {
      font-size: 20px;
    }

    .header .btn {
      width: 100%;
      justify-content: center;
    }

    .recording-content {
      padding: var(--spacing-sm) var(--spacing-md);
      gap: var(--spacing-md);
    }

    .recording-icon {
      width: 36px;
      height: 36px;
    }

    .recording-icon svg {
      width: 18px;
      height: 18px;
    }

    .recording-name {
      font-size: 13px;
    }

    .recording-meta {
      font-size: 11px;
      flex-wrap: wrap;
    }

    .action-btn {
      width: 28px;
      height: 28px;
    }

    .action-btn svg {
      width: 16px;
      height: 16px;
    }
  }

  @media (max-width: 480px) {
    .header h2 {
      font-size: 18px;
    }

    .recording-content {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }

    .recording-main {
      width: 100%;
    }

    .recording-actions {
      width: 100%;
      justify-content: flex-end;
      padding-top: var(--spacing-xs);
      border-top: 1px solid var(--stroke-surface);
    }

    .action-btn {
      width: 36px;
      height: 36px;
    }

    .action-btn svg {
      width: 18px;
      height: 18px;
    }

    .format-badge {
      font-size: 9px;
      padding: 2px 4px;
    }

    .empty-state {
      padding: var(--spacing-xxl) var(--spacing-md);
    }

    .empty-state svg {
      width: 48px;
      height: 48px;
    }

    .empty-state p {
      font-size: 16px;
    }

    .empty-state small {
      font-size: 13px;
    }

    .transcription-progress {
      padding: var(--spacing-sm) var(--spacing-md);
    }
  }
</style>
