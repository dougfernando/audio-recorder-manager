<script>
  import { invoke } from '@tauri-apps/api/tauri';
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
    <label class="form-label">Duration</label>
    <div class="duration-presets">
      {#each durationPresets as preset}
        <button
          class="preset-btn {$selectedDuration === preset && !$isManualMode ? 'active' : ''}"
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
        class="preset-btn manual-btn {$isManualMode ? 'active' : ''}"
        on:click={() => {
          isManualMode.set(true);
        }}
        disabled={$isRecording}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        Manual
      </button>
    </div>
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label">Format</label>
      <select class="form-select" bind:value={$selectedFormat} disabled={$isRecording}>
        {#each formats as format}
          <option value={format}>{format.toUpperCase()}</option>
        {/each}
      </select>
    </div>

    <div class="form-group">
      <label class="form-label">Quality</label>
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
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(255, 255, 255, 0.85) 100%);
    border: 1px solid rgba(0, 103, 192, 0.1);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08), 0 2px 6px rgba(0, 103, 192, 0.05);
  }

  .recording-card:hover {
    border-color: rgba(0, 103, 192, 0.15);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.12), 0 4px 8px rgba(0, 103, 192, 0.08);
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

  .duration-presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .preset-btn {
    padding: var(--spacing-sm) var(--spacing-md);
    background-color: var(--card-background-secondary);
    border: 2px solid transparent;
    border-radius: var(--corner-radius-small);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    transition: all 0.2s ease;
  }

  .preset-btn:hover:not(:disabled) {
    background-color: var(--layer-fill-alt);
    border-color: var(--stroke-surface-flyout);
  }

  .preset-btn.active {
    background: linear-gradient(135deg, #E3F2FD 0%, #BBDEFB 100%);
    border-color: var(--accent-default);
    color: var(--accent-default);
    box-shadow: 0 2px 6px rgba(0, 103, 192, 0.15);
    font-weight: 500;
  }

  .manual-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .manual-btn svg {
    flex-shrink: 0;
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
</style>
