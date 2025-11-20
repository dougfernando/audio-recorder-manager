<script>
  import { invoke } from '@tauri-apps/api/core';
  import {
    isRecording,
    selectedDuration,
    selectedFormat,
    selectedQuality,
    isManualMode,
    durationInSeconds,
    currentSession,
    formatDuration,
  } from '../stores';

  const durationPresets = [30, 60, 300, 600, 1800, 3600]; // 30s, 1m, 5m, 10m, 30m, 1h
  const formats = ['wav', 'm4a'];
  const qualities = [
    { value: 'quick', label: 'Quick (16kHz)', description: '2 MB/min' },
    { value: 'standard', label: 'Standard (44.1kHz)', description: '10 MB/min' },
    { value: 'professional', label: 'Professional (48kHz)', description: '11 MB/min' },
    { value: 'high', label: 'High (96kHz)', description: '22 MB/min' },
  ];

  let isStarting = false;
  let errorMessage = '';

  async function startRecording() {
    if ($isRecording) return;

    isStarting = true;
    errorMessage = '';

    try {
      const result = await invoke('start_recording', {
        durationSecs: $durationInSeconds,
        format: $selectedFormat,
        quality: $selectedQuality,
      });

      console.log('Recording started:', result);
      isRecording.set(true);
      currentSession.set(result.session_id);
    } catch (error) {
      console.error('Failed to start recording:', error);
      errorMessage = error;
    } finally {
      isStarting = false;
    }
  }
</script>

<div class="recording-panel">
  <div class="panel-header">
    <h2 class="panel-title">New Recording</h2>
    <p class="panel-subtitle">Configure and start your audio capture</p>
  </div>

  {#if errorMessage}
    <div class="error-message">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      {errorMessage}
    </div>
  {/if}

  <!-- Duration Selection -->
  <div class="setting-group">
    <label class="setting-label">Duration</label>
    <div class="duration-pills">
      {#each durationPresets as preset}
        <button
          class="duration-pill {$selectedDuration === preset && !$isManualMode ? 'active' : ''}"
          on:click={() => {
            selectedDuration.set(preset);
            isManualMode.set(false);
          }}
          disabled={$isRecording}
        >
          {formatDuration(preset)}
        </button>
      {/each}
      <button
        class="duration-pill manual-pill {$isManualMode ? 'active' : ''}"
        on:click={() => {
          isManualMode.set(true);
        }}
        disabled={$isRecording}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="16"/>
        </svg>
        Manual
      </button>
    </div>
  </div>

  <!-- Format & Quality in compact row -->
  <div class="settings-row">
    <div class="setting-group compact">
      <label class="setting-label" for="format-select">Format</label>
      <select id="format-select" class="setting-select" bind:value={$selectedFormat} disabled={$isRecording}>
        {#each formats as format}
          <option value={format}>{format.toUpperCase()}</option>
        {/each}
      </select>
    </div>

    <div class="setting-group compact">
      <label class="setting-label" for="quality-select">Quality</label>
      <select id="quality-select" class="setting-select" bind:value={$selectedQuality} disabled={$isRecording}>
        {#each qualities as quality}
          <option value={quality.value}>{quality.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Quality info hint -->
  <div class="quality-hint">
    {#each qualities as quality}
      {#if quality.value === $selectedQuality}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4M12 8h.01"/>
        </svg>
        Estimated size: {quality.description}
      {/if}
    {/each}
  </div>

  <!-- Start Button -->
  <button
    class="start-recording-btn {$isRecording ? 'recording' : ''}"
    on:click={startRecording}
    disabled={$isRecording || isStarting}
  >
    {#if isStarting}
      <div class="btn-spinner"></div>
      <span>Starting...</span>
    {:else if $isRecording}
      <div class="recording-pulse"></div>
      <span>Recording Active</span>
    {:else}
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="12" cy="12" r="8"/>
      </svg>
      <span>Start Recording</span>
    {/if}
  </button>
</div>

<style>
  .recording-panel {
    background: var(--card-background);
    border: 1px solid var(--stroke-surface);
    border-radius: var(--corner-radius-large);
    padding: var(--spacing-xxl);
    box-shadow: var(--shadow-sm);
    transition: all 0.2s ease;
  }

  .recording-panel:hover {
    box-shadow: var(--shadow-md);
    border-color: var(--accent-default);
  }

  .panel-header {
    margin-bottom: var(--spacing-xxl);
  }

  .panel-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
  }

  .panel-subtitle {
    font-size: 13px;
    color: var(--text-tertiary);
    margin: 0;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    background-color: rgba(255, 59, 48, 0.1);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
    font-size: 13px;
  }

  .error-message svg {
    flex-shrink: 0;
  }

  .setting-group {
    margin-bottom: var(--spacing-lg);
  }

  .setting-group.compact {
    margin-bottom: 0;
  }

  .setting-label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-sm);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .duration-pills {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .duration-pill {
    padding: 8px 16px;
    background: var(--card-background-secondary);
    border: 1px solid var(--stroke-surface);
    border-radius: 20px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .duration-pill:hover:not(:disabled) {
    background: var(--card-background);
    border-color: var(--accent-default);
    color: var(--text-primary);
    transform: translateY(-1px);
  }

  .duration-pill.active {
    background: var(--accent-default);
    border-color: var(--accent-default);
    color: white;
  }

  .duration-pill.active:hover:not(:disabled) {
    background: var(--accent-secondary);
    border-color: var(--accent-secondary);
  }

  .duration-pill:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .manual-pill {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .manual-pill svg {
    flex-shrink: 0;
  }

  .settings-row {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-sm);
  }

  .setting-select {
    width: 100%;
    padding: 10px 12px;
    background: var(--card-background-secondary);
    border: 1px solid var(--stroke-surface);
    border-radius: var(--corner-radius-medium);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .setting-select:hover:not(:disabled) {
    border-color: var(--accent-default);
  }

  .setting-select:focus {
    outline: none;
    border-color: var(--accent-default);
    box-shadow: 0 0 0 3px rgba(0, 103, 192, 0.1);
  }

  .setting-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .quality-hint {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: 11px;
    color: var(--text-tertiary);
    margin-bottom: var(--spacing-xxl);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--card-background-secondary);
    border-radius: var(--corner-radius-small);
  }

  .quality-hint svg {
    flex-shrink: 0;
    opacity: 0.6;
  }

  .start-recording-btn {
    width: 100%;
    padding: 16px 24px;
    background: linear-gradient(135deg, #FF3B30 0%, #FF6B6B 100%);
    border: none;
    border-radius: var(--corner-radius-medium);
    color: white;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    box-shadow: 0 4px 12px rgba(255, 59, 48, 0.3);
  }

  .start-recording-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(255, 59, 48, 0.4);
  }

  .start-recording-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .start-recording-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .start-recording-btn.recording {
    background: linear-gradient(135deg, #4CAF50 0%, #66BB6A 100%);
  }

  .btn-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .recording-pulse {
    width: 12px;
    height: 12px;
    background: white;
    border-radius: 50%;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(0.8);
    }
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .recording-panel {
      padding: var(--spacing-lg);
    }

    .panel-title {
      font-size: 18px;
    }

    .settings-row {
      grid-template-columns: 1fr;
      gap: var(--spacing-md);
      margin-bottom: var(--spacing-md);
    }

    .setting-group {
      margin-bottom: var(--spacing-md);
    }

    .duration-pills {
      gap: var(--spacing-xs);
    }

    .duration-pill {
      padding: 6px 12px;
      font-size: 12px;
    }
  }

  @media (max-width: 480px) {
    .recording-panel {
      padding: var(--spacing-md);
    }

    .panel-header {
      margin-bottom: var(--spacing-lg);
    }

    .panel-title {
      font-size: 16px;
    }

    .panel-subtitle {
      font-size: 12px;
    }

    .duration-pills {
      justify-content: stretch;
    }

    .duration-pill {
      flex: 1 1 auto;
      min-width: 0;
      font-size: 11px;
      padding: 6px 10px;
    }

    .start-recording-btn {
      font-size: 14px;
      padding: 14px 20px;
    }
  }
</style>
