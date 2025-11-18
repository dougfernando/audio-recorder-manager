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

<div class="card recording-card">
  <h2 class="card-title">Start Recording</h2>

  {#if errorMessage}
    <div class="error-message">
      {errorMessage}
    </div>
  {/if}

  <div class="form-group">
    <label class="form-label">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      Duration
    </label>
    <div class="duration-grid">
      {#each durationPresets as preset}
        <button
          class="duration-btn {$selectedDuration === preset && !$isManualMode ? 'active' : ''}"
          on:click={() => {
            selectedDuration.set(preset);
            isManualMode.set(false);
          }}
          disabled={$isRecording}
        >
          <span class="duration-value">{formatDuration(preset)}</span>
        </button>
      {/each}
      <button
        class="duration-btn manual-mode-btn {$isManualMode ? 'active' : ''}"
        on:click={() => {
          isManualMode.set(true);
        }}
        disabled={$isRecording}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="16"/>
        </svg>
        <span class="duration-value">Manual</span>
      </button>
    </div>
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="12" y1="18" x2="12" y2="12"/>
          <line x1="9" y1="15" x2="15" y2="15"/>
        </svg>
        Format
      </label>
      <select class="form-select" bind:value={$selectedFormat} disabled={$isRecording}>
        {#each formats as format}
          <option value={format}>{format.toUpperCase()}</option>
        {/each}
      </select>
    </div>

    <div class="form-group">
      <label class="form-label">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
        </svg>
        Quality
      </label>
      <select class="form-select" bind:value={$selectedQuality} disabled={$isRecording}>
        {#each qualities as quality}
          <option value={quality.value}>{quality.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <button
    class="btn btn-primary btn-lg start-btn"
    on:click={startRecording}
    disabled={$isRecording || isStarting}
  >
    {#if isStarting}
      Starting...
    {:else if $isRecording}
      Recording in Progress
    {:else}
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3" fill="currentColor"/>
      </svg>
      Start Recording
    {/if}
  </button>
</div>

<style>
  .recording-card {
    background:
      var(--bg-gradient-purple),
      linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(255, 255, 255, 0.85) 100%);
    border: 1px solid rgba(102, 126, 234, 0.2);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08), 0 2px 6px rgba(102, 126, 234, 0.1);
  }

  .recording-card:hover {
    border-color: rgba(102, 126, 234, 0.3);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.12), 0 4px 8px rgba(102, 126, 234, 0.15);
  }

  .error-message {
    background-color: var(--danger-bg);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
    font-size: 14px;
  }

  .form-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .form-label svg {
    opacity: 0.9;
    color: #667eea;
  }

  .duration-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .duration-btn {
    padding: var(--spacing-lg);
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.9) 0%, rgba(249, 249, 249, 0.85) 100%);
    border: 2px solid rgba(0, 103, 192, 0.08);
    border-radius: var(--corner-radius-medium);
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    transition: all 0.15s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.04);
    min-height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .duration-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(255, 255, 255, 1) 0%, rgba(249, 249, 249, 0.95) 100%);
    border-color: rgba(0, 103, 192, 0.2);
    box-shadow: 0 4px 12px rgba(0, 103, 192, 0.1);
    transform: translateY(-2px);
  }

  .duration-btn.active {
    background: linear-gradient(135deg, #0067C0 0%, #0078D4 100%);
    border-color: transparent;
    color: var(--text-on-accent);
    box-shadow: 0 4px 16px rgba(0, 103, 192, 0.35), 0 0 0 1px rgba(0, 103, 192, 0.1);
    font-weight: 700;
  }

  .duration-btn.active:hover:not(:disabled) {
    background: linear-gradient(135deg, #0078D4 0%, #4A9EFF 100%);
    box-shadow: 0 6px 20px rgba(0, 103, 192, 0.45);
  }

  .duration-value {
    font-size: 15px;
    line-height: 1;
  }

  .manual-mode-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
  }

  .manual-mode-btn svg {
    flex-shrink: 0;
    stroke-width: 2.5;
  }

  .manual-mode-btn.active svg {
    stroke: var(--text-on-accent);
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .start-btn {
    width: 100%;
    background: linear-gradient(135deg, #0067C0 0%, #0078D4 100%);
    box-shadow: 0 4px 12px rgba(0, 103, 192, 0.3);
    transition: all 0.2s ease;
  }

  .start-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, #0078D4 0%, #4A9EFF 100%);
    box-shadow: 0 6px 16px rgba(0, 103, 192, 0.4);
    transform: translateY(-1px);
  }

  .start-btn:active:not(:disabled) {
    transform: translateY(0);
    box-shadow: 0 2px 8px rgba(0, 103, 192, 0.3);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .duration-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: var(--spacing-sm);
    }

    .duration-btn {
      padding: var(--spacing-md);
      min-height: 48px;
      font-size: 14px;
    }

    .duration-value {
      font-size: 14px;
    }

    .form-row {
      grid-template-columns: 1fr;
      gap: var(--spacing-md);
    }
  }

  @media (max-width: 480px) {
    .recording-card {
      padding: var(--spacing-md);
    }

    .card-title {
      font-size: 16px;
    }

    .duration-grid {
      gap: var(--spacing-xs);
    }

    .duration-btn {
      padding: var(--spacing-sm);
      min-height: 44px;
      font-size: 13px;
    }

    .duration-value {
      font-size: 13px;
    }

    .form-label {
      font-size: 12px;
      margin-bottom: var(--spacing-sm);
    }

    .form-select {
      font-size: 13px;
    }

    .start-btn {
      font-size: 14px;
      padding: var(--spacing-md) var(--spacing-lg);
    }
  }

  /* Dark mode adjustments */
  @media (prefers-color-scheme: dark) {
    .recording-card {
      background:
        var(--bg-gradient-purple),
        linear-gradient(135deg, rgba(42, 42, 42, 0.95) 0%, rgba(36, 36, 36, 0.85) 100%);
      border: 1px solid rgba(102, 126, 234, 0.3);
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.28), 0 2px 6px rgba(102, 126, 234, 0.15);
    }

    .recording-card:hover {
      border-color: rgba(102, 126, 234, 0.4);
      box-shadow: 0 12px 32px rgba(0, 0, 0, 0.38), 0 4px 8px rgba(102, 126, 234, 0.2);
    }
  }
</style>
