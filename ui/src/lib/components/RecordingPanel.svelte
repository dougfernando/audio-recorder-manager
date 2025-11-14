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

<div class="card">
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
    </div>
    <div class="manual-mode">
      <label class="checkbox-label">
        <input
          type="checkbox"
          bind:checked={$isManualMode}
          disabled={$isRecording}
        />
        <span>Manual Mode (stop manually)</span>
      </label>
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

  <div class="quality-info">
    {#each qualities as quality}
      {#if quality.value === $selectedQuality}
        <small>{quality.description} • {quality.description}</small>
      {/if}
    {/each}
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
  .error-message {
    background-color: #fff2f0;
    border: 1px solid #ffccc7;
    color: var(--danger-color);
    padding: 12px;
    border-radius: var(--radius-md);
    margin-bottom: 16px;
    font-size: 14px;
  }

  .duration-presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-bottom: 12px;
  }

  .preset-btn {
    padding: 10px;
    background-color: var(--bg-secondary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    transition: all 0.2s ease;
  }

  .preset-btn:hover:not(:disabled) {
    background-color: var(--bg-tertiary);
  }

  .preset-btn.active {
    background-color: #e6f7ff;
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .manual-mode {
    margin-top: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .quality-info {
    margin-bottom: 16px;
    padding: 8px 12px;
    background-color: var(--bg-secondary);
    border-radius: var(--radius-md);
    font-size: 13px;
    color: var(--text-secondary);
    text-align: center;
  }

  .start-btn {
    width: 100%;
  }
</style>
