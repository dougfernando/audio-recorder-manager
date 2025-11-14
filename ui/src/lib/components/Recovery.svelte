<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';
  import { recoveryList } from '../stores';

  let isScanning = false;
  let isRecovering = false;
  let recoveringSession = null;
  let selectedFormat = 'wav';
  let recoveryMessage = '';
  let recoveryError = '';

  onMount(async () => {
    await scanForRecovery();
  });

  async function scanForRecovery() {
    isScanning = true;
    try {
      // In a real implementation, we would have a backend command to list incomplete sessions
      // For now, we'll just clear the list
      recoveryList.set([]);
    } catch (error) {
      console.error('Failed to scan for recovery:', error);
    } finally {
      isScanning = false;
    }
  }

  async function recoverSession(sessionId = null) {
    isRecovering = true;
    recoveringSession = sessionId;
    recoveryMessage = '';
    recoveryError = '';

    try {
      const result = await invoke('recover_recordings', {
        sessionId: sessionId,
        format: selectedFormat,
      });

      console.log('Recovery result:', result);
      recoveryMessage = result.message || 'Recovery completed successfully';

      // Refresh the recovery list
      await scanForRecovery();
    } catch (error) {
      console.error('Failed to recover recordings:', error);
      recoveryError = error.toString();
    } finally {
      isRecovering = false;
      recoveringSession = null;
    }
  }

  async function recoverAll() {
    await recoverSession(null);
  }
</script>

<div class="recovery-container">
  <div class="header">
    <h2>Recovery</h2>
    <button class="btn btn-secondary" on:click={scanForRecovery} disabled={isScanning}>
      {#if isScanning}
        <svg class="spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
        Scanning...
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
        </svg>
        Scan
      {/if}
    </button>
  </div>

  <div class="card info-card">
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <line x1="12" y1="16" x2="12" y2="12"/>
      <line x1="12" y1="8" x2="12.01" y2="8"/>
    </svg>
    <div>
      <h3>About Recovery</h3>
      <p>
        If a recording was interrupted (e.g., program crash, system shutdown),
        the temporary audio files are preserved. Use the recovery feature to complete
        the merge and conversion process.
      </p>
    </div>
  </div>

  {#if recoveryMessage}
    <div class="message success-message">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
        <polyline points="22 4 12 14.01 9 11.01"/>
      </svg>
      {recoveryMessage}
    </div>
  {/if}

  {#if recoveryError}
    <div class="message error-message">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="15" y1="9" x2="9" y2="15"/>
        <line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
      {recoveryError}
    </div>
  {/if}

  <div class="card recovery-options">
    <h3>Recovery Options</h3>
    <div class="form-group">
      <label class="form-label">Output Format</label>
      <select class="form-select" bind:value={selectedFormat} disabled={isRecovering}>
        <option value="wav">WAV</option>
        <option value="m4a">M4A</option>
      </select>
    </div>

    <button
      class="btn btn-primary btn-lg"
      on:click={recoverAll}
      disabled={isRecovering || isScanning}
    >
      {#if isRecovering && !recoveringSession}
        <svg class="spin" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
        Recovering...
      {:else}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
        </svg>
        Recover All Incomplete Recordings
      {/if}
    </button>
  </div>

  {#if $recoveryList && $recoveryList.length > 0}
    <div class="recovery-list">
      <h3>Incomplete Sessions</h3>
      {#each $recoveryList as session}
        <div class="recovery-item card">
          <div class="session-info">
            <div class="session-id">{session.session_id}</div>
            <div class="session-files">
              {session.files.join(', ')}
            </div>
          </div>
          <button
            class="btn btn-primary"
            on:click={() => recoverSession(session.session_id)}
            disabled={isRecovering}
          >
            {#if isRecovering && recoveringSession === session.session_id}
              Recovering...
            {:else}
              Recover
            {/if}
          </button>
        </div>
      {/each}
    </div>
  {:else if !isScanning}
    <div class="empty-state">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
      </svg>
      <p>No incomplete recordings found</p>
      <small>All your recordings have been completed successfully</small>
    </div>
  {/if}
</div>

<style>
  .recovery-container {
    max-width: 800px;
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

  h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 12px;
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

  .info-card {
    display: flex;
    gap: 16px;
    background-color: #e6f7ff;
    border: 1px solid #91d5ff;
    margin-bottom: 20px;
  }

  .info-card svg {
    flex-shrink: 0;
    color: var(--primary-color);
    margin-top: 2px;
  }

  .info-card h3 {
    margin-bottom: 6px;
  }

  .info-card p {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 0;
  }

  .message {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: var(--radius-md);
    margin-bottom: 20px;
    font-size: 14px;
    font-weight: 500;
  }

  .success-message {
    background-color: #f6ffed;
    border: 1px solid #b7eb8f;
    color: var(--success-color);
  }

  .error-message {
    background-color: #fff2f0;
    border: 1px solid #ffccc7;
    color: var(--danger-color);
  }

  .recovery-options {
    margin-bottom: 20px;
  }

  .recovery-list {
    margin-top: 20px;
  }

  .recovery-list h3 {
    margin-bottom: 16px;
  }

  .recovery-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .session-info {
    flex: 1;
  }

  .session-id {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 4px;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .session-files {
    font-size: 12px;
    color: var(--text-secondary);
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
