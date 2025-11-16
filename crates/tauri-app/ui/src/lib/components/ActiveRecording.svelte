<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import {
    isRecording,
    currentSession,
    recordingStatus,
    formatTime,
  } from '../stores';

  let isStopping = false;
  let pollInterval;

  // Reactive statement: start/stop polling based on recording state
  $: {
    // Clear existing interval
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }

    // Start polling if recording
    if ($isRecording && $currentSession) {
      console.log('Starting status polling for session:', $currentSession);
      pollInterval = setInterval(async () => {
        try {
          const status = await invoke('get_recording_status', {
            sessionId: $currentSession
          });
          console.log('Polled status:', status);
          if (status) {
            recordingStatus.set(status);
            // If completed, stop recording state
            if (status.status === 'completed') {
              setTimeout(() => {
                isRecording.set(false);
                currentSession.set(null);
                if (pollInterval) {
                  clearInterval(pollInterval);
                }
              }, 5000); // Show completion for 5 seconds
            }
          }
        } catch (error) {
          console.error('Failed to poll recording status:', error);
        }
      }, 1000);
    }
  }

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });

  async function stopRecording() {
    if (!$isRecording) return;

    isStopping = true;
    try {
      await invoke('stop_recording', {
        sessionId: $currentSession,
      });

      console.log('Stop signal sent, waiting for processing to complete...');
      // Don't clear the states here - let the polling continue
      // so it can detect the "processing" and "completed" status updates
      // The polling will auto-stop when it detects status === 'completed'
    } catch (error) {
      console.error('Failed to stop recording:', error);
      // On error, clean up
      isRecording.set(false);
      currentSession.set(null);
      recordingStatus.set(null);
      if (pollInterval) {
        clearInterval(pollInterval);
      }
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

  // Debug logging for frame updates
  $: if (loopbackFrames > 0 || micFrames > 0) {
    console.log('Frame counts updated - Loopback:', loopbackFrames, 'Mic:', micFrames);
  }
</script>

{#if $isRecording && $recordingStatus}
  <!-- Processing Status Header -->
  {#if $recordingStatus.status === 'processing'}
    <div class="processing-header">
      <div class="processing-indicator">
        <div class="spinner"></div>
        <span class="processing-text">PROCESSING</span>
      </div>
      <span class="session-id">{$recordingStatus.session_id || 'N/A'}</span>
    </div>
    <div class="processing-message">
      {$recordingStatus.message || 'Processing audio...'}
    </div>
  {:else if $recordingStatus.status === 'completed'}
    <!-- Completion Header -->
    <div class="completion-header">
      <div class="completion-indicator">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
        <span class="completion-text">COMPLETED</span>
      </div>
      <span class="session-id">{$recordingStatus.session_id || 'N/A'}</span>
    </div>
    <div class="completion-details">
      <div class="completion-message">{$recordingStatus.message || 'Recording completed successfully'}</div>
      {#if $recordingStatus.filename}
        <div class="file-info">
          <strong>File:</strong> {$recordingStatus.filename}
        </div>
      {/if}
      {#if $recordingStatus.file_path}
        <div class="file-path">
          <strong>Location:</strong>
          <code>{$recordingStatus.file_path}</code>
        </div>
      {/if}
      {#if $recordingStatus.file_size_mb}
        <div class="file-size">
          <strong>Size:</strong> {$recordingStatus.file_size_mb} MB
        </div>
      {/if}
    </div>
  {:else}
    <!-- Recording Header with Pulse Animation -->
    <div class="recording-header">
      <div class="recording-indicator">
        <div class="pulse-dot"></div>
        <span class="recording-text">RECORDING</span>
        <span class="live-badge">
          <span class="live-dot"></span>
          LIVE
        </span>
      </div>
      <span class="session-id">{$recordingStatus.session_id || 'N/A'}</span>
    </div>
  {/if}

  <!-- Large Timer Display (only show during recording) -->
  {#if $recordingStatus.status === 'recording'}
  <div class="timer-card">
    <div class="time-display">
      <span class="elapsed">{formatTime(elapsed)}</span>
      {#if duration > 0}
        <span class="separator">/</span>
        <span class="total">{formatTime(duration)}</span>
      {:else}
        <span class="manual-badge">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <circle cx="8" cy="8" r="2"/>
          </svg>
          MANUAL
        </span>
      {/if}
    </div>

    {#if duration > 0}
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progress}%">
          <div class="progress-shine"></div>
        </div>
      </div>
      <div class="progress-label">{progress}% Complete</div>
    {/if}
  </div>

  <!-- Audio Channels with Visual Indicators -->
  <div class="channels-grid">
    <!-- System Audio Card -->
    <div class="channel-card {loopbackHasAudio ? 'active' : 'silent'}">
      <div class="channel-icon system">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
        </svg>
      </div>
      <div class="channel-content">
        <div class="channel-header">
          <span class="channel-name">System Audio</span>
          <span class="channel-badge {loopbackHasAudio ? 'active' : 'silent'}">
            {loopbackHasAudio ? '● ACTIVE' : '○ Silent'}
          </span>
        </div>
        <div class="channel-stats">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3 5h10v6H3V5z"/>
          </svg>
          <span class="frames-count" class:updating={loopbackFrames > 0}>
            {loopbackFrames.toLocaleString()}
          </span>
          <span class="frames-label">frames</span>
        </div>
        {#if loopbackHasAudio}
          <div class="waveform">
            <div class="bar" style="--delay: 0ms"></div>
            <div class="bar" style="--delay: 100ms"></div>
            <div class="bar" style="--delay: 200ms"></div>
            <div class="bar" style="--delay: 150ms"></div>
            <div class="bar" style="--delay: 50ms"></div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Microphone Card -->
    <div class="channel-card {micHasAudio ? 'active' : 'silent'}">
      <div class="channel-icon mic">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 14c1.66 0 2.99-1.34 2.99-3L15 5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.3-3c0 3-2.54 5.1-5.3 5.1S6.7 14 6.7 11H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c3.28-.48 6-3.3 6-6.72h-1.7z"/>
        </svg>
      </div>
      <div class="channel-content">
        <div class="channel-header">
          <span class="channel-name">Microphone</span>
          <span class="channel-badge {micHasAudio ? 'active' : 'silent'}">
            {micHasAudio ? '● ACTIVE' : '○ Silent'}
          </span>
        </div>
        <div class="channel-stats">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3 5h10v6H3V5z"/>
          </svg>
          <span class="frames-count" class:updating={micFrames > 0}>
            {micFrames.toLocaleString()}
          </span>
          <span class="frames-label">frames</span>
        </div>
        {#if micHasAudio}
          <div class="waveform">
            <div class="bar" style="--delay: 50ms"></div>
            <div class="bar" style="--delay: 150ms"></div>
            <div class="bar" style="--delay: 0ms"></div>
            <div class="bar" style="--delay: 200ms"></div>
            <div class="bar" style="--delay: 100ms"></div>
          </div>
        {/if}
      </div>
    </div>
  </div>
  {/if}

  <!-- Stop Button (only show during recording) -->
  {#if $recordingStatus.status === 'recording'}
  <button
    class="btn btn-danger btn-lg stop-btn"
    on:click={stopRecording}
    disabled={isStopping}
  >
    {#if isStopping}
      <svg class="spin" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <circle cx="12" cy="12" r="2"/>
      </svg>
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
  <!-- Idle State -->
  <div class="idle-state">
    <div class="idle-icon">
      <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="6" fill="currentColor" opacity="0.1"/>
        <circle cx="12" cy="12" r="3" fill="currentColor"/>
      </svg>
    </div>
    <h3>Ready to Record</h3>
    <p>Configure your settings and click "Start Recording"</p>
  </div>
{/if}

<style>
  /* Recording Header */
  .recording-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg);
    background: linear-gradient(135deg, var(--danger) 0%, #E02020 100%);
    border-radius: var(--corner-radius-medium) var(--corner-radius-medium) 0 0;
    margin: calc(var(--spacing-lg) * -1) calc(var(--spacing-lg) * -1) var(--spacing-lg);
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .pulse-dot {
    width: 12px;
    height: 12px;
    background-color: white;
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.3;
      transform: scale(0.8);
    }
  }

  .recording-text {
    color: white;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: 1px;
  }

  .live-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px var(--spacing-md);
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 12px;
    color: white;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.5px;
  }

  .live-dot {
    width: 6px;
    height: 6px;
    background-color: #4ade80;
    border-radius: 50%;
    animation: livePulse 2s ease-in-out infinite;
  }

  @keyframes livePulse {
    0%, 100% {
      opacity: 1;
      box-shadow: 0 0 0 0 rgba(74, 222, 128, 0.7);
    }
    50% {
      opacity: 0.7;
      box-shadow: 0 0 0 4px rgba(74, 222, 128, 0);
    }
  }

  .session-id {
    color: rgba(255, 255, 255, 0.9);
    font-size: 13px;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  /* Timer Card */
  .timer-card {
    background-color: var(--card-background-secondary);
    border-radius: var(--corner-radius-medium);
    padding: var(--spacing-xxl);
    margin-bottom: var(--spacing-lg);
    text-align: center;
  }

  .time-display {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    font-size: 48px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .elapsed {
    color: var(--accent-default);
  }

  .separator {
    color: var(--text-tertiary);
    font-weight: 400;
  }

  .total {
    color: var(--text-secondary);
  }

  .manual-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-lg);
    background-color: var(--warning-bg);
    color: var(--warning);
    border-radius: var(--corner-radius-small);
    font-size: 16px;
    font-weight: 600;
  }

  .progress-bar {
    height: 12px;
    background-color: var(--layer-fill-alt);
    border-radius: 6px;
    overflow: hidden;
    margin-bottom: var(--spacing-sm);
    position: relative;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-default), var(--accent-secondary));
    transition: width 0.5s ease;
    position: relative;
    overflow: hidden;
  }

  .progress-shine {
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
    animation: shine 2s infinite;
  }

  @keyframes shine {
    to {
      left: 100%;
    }
  }

  .progress-label {
    font-size: 14px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  /* Channels Grid */
  .channels-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .channel-card {
    background-color: var(--card-background-secondary);
    border-radius: var(--corner-radius-medium);
    padding: var(--spacing-lg);
    border: 2px solid var(--stroke-surface);
    transition: all 0.2s ease;
  }

  .channel-card.active {
    border-color: var(--success);
    box-shadow: 0 0 0 1px var(--success), var(--elevation-card);
  }

  .channel-card.silent {
    opacity: 0.6;
  }

  .channel-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--corner-radius-medium);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--spacing-md);
  }

  .channel-icon.system {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
  }

  .channel-icon.mic {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
    color: white;
  }

  .channel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .channel-name {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .channel-badge {
    font-size: 11px;
    font-weight: 700;
    padding: 4px var(--spacing-sm);
    border-radius: 10px;
    letter-spacing: 0.5px;
  }

  .channel-badge.active {
    background-color: var(--success-bg);
    color: var(--success);
  }

  .channel-badge.silent {
    background-color: var(--layer-fill-alt);
    color: var(--text-tertiary);
  }

  .channel-stats {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: 'Consolas', 'Monaco', monospace;
    margin-bottom: var(--spacing-sm);
  }

  .channel-stats svg {
    opacity: 0.5;
  }

  .frames-count {
    font-weight: 600;
    font-size: 18px;
    color: var(--accent-default);
    transition: all 0.3s ease;
    min-width: 80px;
    text-align: left;
  }

  .frames-count.updating {
    color: var(--success);
    font-weight: 700;
  }

  .frames-label {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  /* Waveform Animation */
  .waveform {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    height: 32px;
  }

  .bar {
    width: 4px;
    background: linear-gradient(180deg, var(--success), var(--success) 50%, transparent 50%);
    border-radius: 2px;
    animation: wave 1s ease-in-out infinite;
    animation-delay: var(--delay);
  }

  @keyframes wave {
    0%, 100% {
      height: 8px;
    }
    50% {
      height: 32px;
    }
  }

  /* Stop Button */
  .stop-btn {
    width: 100%;
  }

  /* Idle State */
  .idle-state {
    text-align: center;
    padding: var(--spacing-xxxl) var(--spacing-lg);
    color: var(--text-tertiary);
  }

  .idle-icon {
    margin-bottom: var(--spacing-lg);
    color: var(--accent-default);
    opacity: 0.3;
  }

  .idle-state h3 {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .idle-state p {
    font-size: 14px;
    color: var(--text-secondary);
  }

  /* Processing Header */
  .processing-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg);
    background: linear-gradient(135deg, var(--warning) 0%, #F59E0B 100%);
    border-radius: var(--corner-radius-medium) var(--corner-radius-medium) 0 0;
    margin: calc(var(--spacing-lg) * -1) calc(var(--spacing-lg) * -1) var(--spacing-lg);
  }

  .processing-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .processing-text {
    color: white;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: 1px;
  }

  .processing-message {
    padding: var(--spacing-lg);
    background-color: var(--warning-bg);
    color: var(--warning);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
    text-align: center;
    font-weight: 600;
  }

  /* Completion Header */
  .completion-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg);
    background: linear-gradient(135deg, var(--success) 0%, #10B981 100%);
    border-radius: var(--corner-radius-medium) var(--corner-radius-medium) 0 0;
    margin: calc(var(--spacing-lg) * -1) calc(var(--spacing-lg) * -1) var(--spacing-lg);
  }

  .completion-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    color: white;
  }

  .completion-text {
    color: white;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: 1px;
  }

  .completion-details {
    padding: var(--spacing-lg);
    background-color: var(--success-bg);
    border: 2px solid var(--success);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
  }

  .completion-message {
    font-size: 16px;
    font-weight: 600;
    color: var(--success);
    margin-bottom: var(--spacing-md);
  }

  .file-info, .file-path, .file-size {
    margin-top: var(--spacing-sm);
    font-size: 14px;
    color: var(--text-secondary);
  }

  .file-info strong, .file-path strong, .file-size strong {
    color: var(--text-primary);
    margin-right: var(--spacing-sm);
  }

  .file-path code {
    display: inline-block;
    margin-top: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    background-color: var(--layer-fill-alt);
    border-radius: var(--corner-radius-small);
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    word-break: break-all;
    color: var(--text-primary);
  }

  /* Responsive */
  @media (max-width: 768px) {
    .channels-grid {
      grid-template-columns: 1fr;
    }

    .time-display {
      font-size: 36px;
    }
  }
</style>
