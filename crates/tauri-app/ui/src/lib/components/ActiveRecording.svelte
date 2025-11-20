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
  /* Recording Header - Explosive & Animated */
  .recording-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-xl);
    background: var(--gradient-recording);
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    margin: calc(var(--spacing-lg) * -1) calc(var(--spacing-lg) * -1) var(--spacing-xl);
    position: relative;
    overflow: hidden;
  }

  .recording-header::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 200%;
    height: 100%;
    background: linear-gradient(90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.1) 50%,
      transparent 100%
    );
    animation: shimmer 3s linear infinite;
  }

  @keyframes shimmer {
    to {
      left: 100%;
    }
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    position: relative;
    z-index: 1;
  }

  .pulse-dot {
    width: 16px;
    height: 16px;
    background-color: white;
    border-radius: 50%;
    position: relative;
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.8);
  }

  .pulse-dot::before,
  .pulse-dot::after {
    content: '';
    position: absolute;
    inset: -8px;
    border: 2px solid white;
    border-radius: 50%;
    opacity: 0;
  }

  .pulse-dot::before {
    animation: pulseRing 2s ease-out infinite;
  }

  .pulse-dot::after {
    animation: pulseRing 2s ease-out infinite 1s;
  }

  @keyframes pulseRing {
    0% {
      opacity: 1;
      transform: scale(0.5);
    }
    100% {
      opacity: 0;
      transform: scale(1.5);
    }
  }

  .recording-text {
    color: white;
    font-size: 18px;
    font-weight: 900;
    letter-spacing: 0.15em;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
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

  /* Timer Card - Large & Monospace */
  .timer-card {
    background: var(--bg-surface);
    border: 2px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xxxl);
    margin-bottom: var(--spacing-xl);
    text-align: center;
    position: relative;
    overflow: hidden;
  }

  .timer-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--gradient-recording);
  }

  .time-display {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-lg);
    font-size: 72px;
    font-weight: 700;
    margin-bottom: var(--spacing-xl);
    font-family: 'IBM Plex Mono', monospace;
    line-height: 1;
  }

  .elapsed {
    color: var(--rec-active);
    text-shadow: 0 0 24px var(--accent-magenta-glow);
    animation: numberPulse 2s ease-in-out infinite;
  }

  @keyframes numberPulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.02); }
  }

  .separator {
    color: var(--text-tertiary);
    font-weight: 300;
    opacity: 0.5;
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
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: var(--spacing-md);
    position: relative;
    border: 1px solid var(--border-subtle);
  }

  .progress-fill {
    height: 100%;
    background: var(--gradient-recording);
    transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 0 0 12px var(--accent-magenta-glow);
  }

  .progress-fill::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.3) 50%,
      transparent 100%
    );
    animation: shine 2s linear infinite;
  }

  @keyframes shine {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(200%);
    }
  }

  .progress-label {
    font-size: 14px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  /* Channels Grid - Elevated Cards */
  .channels-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);
  }

  .channel-card {
    background: var(--gradient-surface);
    border-radius: var(--radius-md);
    padding: var(--spacing-xl);
    border: 2px solid var(--border-subtle);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
  }

  .channel-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: var(--success);
    opacity: 0;
    transition: opacity 0.3s;
  }

  .channel-card.active {
    border-color: var(--success);
    box-shadow: var(--shadow-md), 0 0 24px rgba(0, 255, 159, 0.2);
    transform: translateY(-2px);
  }

  .channel-card.active::before {
    opacity: 1;
  }

  .channel-card.silent {
    opacity: 0.5;
  }

  .channel-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--spacing-lg);
    box-shadow: var(--shadow-sm);
  }

  .channel-icon.system {
    background: linear-gradient(135deg, var(--accent-cyan) 0%, var(--accent-magenta) 100%);
    color: var(--text-on-accent);
  }

  .channel-icon.mic {
    background: linear-gradient(135deg, var(--accent-magenta) 0%, var(--rec-active) 100%);
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
    .recording-header,
    .processing-header,
    .completion-header {
      padding: var(--spacing-md);
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }

    .recording-text,
    .processing-text,
    .completion-text {
      font-size: 14px;
    }

    .session-id {
      font-size: 12px;
    }

    .channels-grid {
      grid-template-columns: 1fr;
      gap: var(--spacing-md);
    }

    .time-display {
      font-size: 36px;
    }

    .timer-card {
      padding: var(--spacing-lg);
    }

    .channel-card {
      padding: var(--spacing-md);
    }

    .channel-icon {
      width: 40px;
      height: 40px;
    }

    .channel-icon svg {
      width: 20px;
      height: 20px;
    }

    .frames-count {
      font-size: 16px;
      min-width: 60px;
    }

    .completion-details {
      padding: var(--spacing-md);
    }

    .file-path code {
      font-size: 12px;
      word-break: break-all;
    }
  }

  @media (max-width: 480px) {
    .recording-header,
    .processing-header,
    .completion-header {
      padding: var(--spacing-sm);
      margin: calc(var(--spacing-md) * -1) calc(var(--spacing-md) * -1) var(--spacing-md);
    }

    .recording-indicator,
    .processing-indicator,
    .completion-indicator {
      gap: var(--spacing-sm);
    }

    .pulse-dot {
      width: 10px;
      height: 10px;
    }

    .recording-text,
    .processing-text,
    .completion-text {
      font-size: 13px;
      letter-spacing: 0.5px;
    }

    .live-badge {
      padding: 3px var(--spacing-sm);
      font-size: 10px;
    }

    .live-dot {
      width: 5px;
      height: 5px;
    }

    .session-id {
      font-size: 11px;
    }

    .timer-card {
      padding: var(--spacing-md);
    }

    .time-display {
      font-size: 28px;
      gap: var(--spacing-sm);
    }

    .manual-badge {
      font-size: 14px;
      padding: var(--spacing-xs) var(--spacing-md);
    }

    .progress-label {
      font-size: 13px;
    }

    .channels-grid {
      gap: var(--spacing-sm);
    }

    .channel-card {
      padding: var(--spacing-sm);
    }

    .channel-icon {
      width: 36px;
      height: 36px;
      margin-bottom: var(--spacing-sm);
    }

    .channel-icon svg {
      width: 18px;
      height: 18px;
    }

    .channel-name {
      font-size: 13px;
    }

    .channel-badge {
      font-size: 10px;
      padding: 3px var(--spacing-xs);
    }

    .frames-count {
      font-size: 14px;
      min-width: 50px;
    }

    .frames-label {
      font-size: 11px;
    }

    .waveform {
      height: 24px;
    }

    .completion-message {
      font-size: 14px;
    }

    .file-info,
    .file-path,
    .file-size {
      font-size: 13px;
    }

    .file-path code {
      font-size: 11px;
    }

    .processing-message {
      padding: var(--spacing-md);
      font-size: 13px;
    }

    .idle-state {
      padding: var(--spacing-xxl) var(--spacing-md);
    }

    .idle-icon svg {
      width: 64px;
      height: 64px;
    }

    .idle-state h3 {
      font-size: 18px;
    }

    .idle-state p {
      font-size: 13px;
    }

    .stop-btn {
      font-size: 14px;
      padding: var(--spacing-md) var(--spacing-lg);
    }
  }
</style>
