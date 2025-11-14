<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { open } from '@tauri-apps/api/shell';
  import { recordings, formatFileSize } from '../stores';
  import { onMount } from 'svelte';

  let isLoading = false;
  let isDeleting = {};

  onMount(async () => {
    await loadRecordings();
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
    try {
      await open(path);
    } catch (error) {
      console.error('Failed to open recording:', error);
    }
  }

  async function deleteRecording(recording) {
    if (!confirm(`Are you sure you want to delete "${recording.filename}"?`)) {
      return;
    }

    isDeleting[recording.filename] = true;
    try {
      // Note: This functionality would need to be implemented in the backend
      // For now, we'll just show the UI
      console.log('Delete not implemented yet:', recording.path);
      await loadRecordings();
    } catch (error) {
      console.error('Failed to delete recording:', error);
    } finally {
      isDeleting[recording.filename] = false;
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
            <div class="recording-icon">
              {#if recording.format === 'wav'}
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 18V5l12-2v13"/>
                  <circle cx="6" cy="18" r="3"/>
                  <circle cx="18" cy="16" r="3"/>
                </svg>
              {:else}
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 18V5l12-2v13"/>
                  <circle cx="6" cy="18" r="3"/>
                  <circle cx="18" cy="16" r="3"/>
                </svg>
              {/if}
            </div>
            <div class="format-badge">{recording.format.toUpperCase()}</div>
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
              class="btn btn-secondary btn-sm"
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

<style>
  .recordings-container {
    max-width: 1200px;
    margin: 0 auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
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
    gap: 20px;
  }

  .recording-card {
    display: flex;
    flex-direction: column;
    transition: all 0.2s ease;
  }

  .recording-card:hover {
    box-shadow: var(--shadow-lg);
    transform: translateY(-2px);
  }

  .recording-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .recording-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    background: linear-gradient(135deg, var(--primary-color), #357abd);
    border-radius: var(--radius-lg);
    color: white;
  }

  .format-badge {
    padding: 4px 10px;
    background-color: var(--bg-secondary);
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .recording-info {
    flex: 1;
    margin-bottom: 16px;
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
  }

  .recording-actions {
    display: flex;
    gap: 8px;
  }

  .btn-sm {
    padding: 8px 14px;
    font-size: 13px;
    flex: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 20px;
    color: var(--text-tertiary);
  }

  .empty-state svg {
    margin-bottom: 16px;
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
