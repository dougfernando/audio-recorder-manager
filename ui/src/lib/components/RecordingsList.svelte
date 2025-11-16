<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { ask } from '@tauri-apps/api/dialog';
  import { recordings, formatFileSize } from '../stores';
  import { onMount } from 'svelte';
  import TranscriptViewer from './TranscriptViewer.svelte';

  let isLoading = false;
  let isDeleting = {};
  let isTranscribing = {};
  let transcriptPath = {};
  let viewingTranscript = null; // { path: string, name: string }

  onMount(async () => {
    await loadRecordings();
    checkForTranscripts();
  });

  async function loadRecordings() {
    isLoading = true;
    try {
      const result = await invoke('list_recordings');
      recordings.set(result);
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

    try {
      isTranscribing[recording.filename] = true;
      isTranscribing = isTranscribing; // Trigger reactivity

      const result = await invoke('transcribe_recording', {
        filePath: recording.path,
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

  function checkForTranscripts() {
    // Check if .md files exist for each recording
    $recordings.forEach((recording) => {
      const mdPath = recording.path.replace(/\.(wav|m4a)$/, '.md');
      // We can't easily check file existence from frontend, so this would need backend support
      // For now, we'll just enable the button and let the backend handle it
    });
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
            {#if transcriptPath[recording.filename]}
              <button
                class="btn btn-info btn-sm"
                on:click={() => viewTranscript(recording)}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                View Transcript
              </button>
            {/if}
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
        </div>
      {/each}
    </div>
  {:else if isLoading}
    <div class="empty-state">
      <svg class="spin" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 11-6.219-8.56"/>
      </svg>
      <p>Loading recordings...</p>
    </div>
  {:else}
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
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--spacing-lg);
  }

  .recording-card {
    display: flex;
    flex-direction: column;
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
    width: 56px;
    height: 56px;
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
    display: flex;
    gap: var(--spacing-sm);
  }

  .btn-sm {
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: 13px;
    flex: 1;
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
</style>
