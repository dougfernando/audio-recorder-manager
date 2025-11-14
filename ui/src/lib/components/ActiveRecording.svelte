<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import {
    isRecording,
    currentSession,
    recordingStatus,
    formatTime,
  } from '../stores';

  let isStopping = false;

  async function stopRecording() {
    if (!$isRecording) return;

    isStopping = true;
    try {
      await invoke('stop_recording', {
        sessionId: $currentSession,
      });

      console.log('Recording stopped');
      isRecording.set(false);
      currentSession.set(null);
      recordingStatus.set(null);
    } catch (error) {
      console.error('Failed to stop recording:', error);
    } finally {
      isStopping = false;
    }
  }

  $: progress = $recordingStatus?.progress || 0;
  $: elapsed = $recordingStatus?.elapsed || 0;
  $: duration = $recordingStatus?.duration || 0;
  $: hasAudio = $recordingStatus?.has_audio || false;
  $: loopbackFrames = $recordingStatus?.loopback_frames || 0;
  $: loopbackHasAudio = $recordingStatus?.loopback_has_audio || false;
  $: micFrames = $recordingStatus?.mic_frames || 0;
  $: micHasAudio = $recordingStatus?.mic_has_audio || false;
</script>

<div class="card">
  <h2 class="card-title">
    {#if $isRecording}
      <span class="status-badge status-recording">Recording</span>
    {:else}
      <span class="status-badge status-idle">Idle</span>
    {/if}
  </h2>

  {#if $isRecording && $recordingStatus}
    <div class="recording-info">
      <div class="info-row">
        <span class="label">Session ID:</span>
        <span class="value">{$recordingStatus.session_id || 'N/A'}</span>
      </div>
      <div class="info-row">
        <span class="label">Filename:</span>
        <span class="value">{$recordingStatus.filename || 'N/A'}</span>
      </div>
    </div>

    <div class="progress-section">
      <div class="time-display">
        <span class="elapsed">{formatTime(elapsed)}</span>
        {#if duration > 0}
          <span class="separator">/</span>
          <span class="total">{formatTime(duration)}</span>
        {:else}
          <span class="separator">•</span>
          <span class="manual">Manual Mode</span>
        {/if}
      </div>

      {#if duration > 0}
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress}%"></div>
        </div>
        <div class="progress-text">{progress}%</div>
      {/if}
    </div>

    <div class="channels-section">
      <h3>Audio Channels</h3>
      <div class="channel">
        <div class="channel-header">
          <span class="channel-name">System Audio (Loopback)</span>
          <span class="channel-status {loopbackHasAudio ? 'active' : 'silent'}">
            {loopbackHasAudio ? 'Active' : 'Silent'}
          </span>
        </div>
        <div class="channel-info">
          <span class="frames">{loopbackFrames.toLocaleString()} frames</span>
        </div>
      </div>

      <div class="channel">
        <div class="channel-header">
          <span class="channel-name">Microphone</span>
          <span class="channel-status {micHasAudio ? 'active' : 'silent'}">
            {micHasAudio ? 'Active' : 'Silent'}
          </span>
        </div>
        <div class="channel-info">
          <span class="frames">{micFrames.toLocaleString()} frames</span>
        </div>
      </div>
    </div>

    {#if $recordingStatus.status === 'recording'}
      <button
        class="btn btn-danger btn-lg stop-btn"
        on:click={stopRecording}
        disabled={isStopping}
      >
        {#if isStopping}
          Stopping...
        {:else}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" rx="2"/>
          </svg>
          Stop Recording
        {/if}
      </button>
    {/if}
  {:else}
    <div class="idle-state">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
      <p>No active recording</p>
      <small>Start a recording to see live status</small>
    </div>
  {/if}
</div>

<style>
  .recording-info {
    margin-bottom: 20px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .label {
    font-weight: 500;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .value {
    color: var(--text-primary);
    font-size: 14px;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .progress-section {
    margin-bottom: 24px;
  }

  .time-display {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 32px;
    font-weight: 600;
    margin-bottom: 16px;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .elapsed {
    color: var(--primary-color);
  }

  .separator {
    color: var(--text-tertiary);
  }

  .total {
    color: var(--text-secondary);
  }

  .manual {
    color: var(--warning-color);
    font-size: 18px;
  }

  .progress-bar {
    height: 8px;
    background-color: var(--bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--primary-color), #357abd);
    transition: width 0.3s ease;
  }

  .progress-text {
    text-align: center;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .channels-section {
    margin-bottom: 24px;
  }

  .channels-section h3 {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--text-primary);
  }

  .channel {
    padding: 12px;
    background-color: var(--bg-secondary);
    border-radius: var(--radius-md);
    margin-bottom: 8px;
  }

  .channel:last-child {
    margin-bottom: 0;
  }

  .channel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }

  .channel-name {
    font-weight: 500;
    font-size: 14px;
    color: var(--text-primary);
  }

  .channel-status {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 10px;
  }

  .channel-status.active {
    background-color: #f6ffed;
    color: var(--success-color);
  }

  .channel-status.silent {
    background-color: var(--bg-tertiary);
    color: var(--text-tertiary);
  }

  .channel-info {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .frames {
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .stop-btn {
    width: 100%;
  }

  .idle-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: var(--text-tertiary);
  }

  .idle-state svg {
    margin-bottom: 16px;
    opacity: 0.3;
  }

  .idle-state p {
    font-size: 16px;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .idle-state small {
    font-size: 13px;
  }
</style>
