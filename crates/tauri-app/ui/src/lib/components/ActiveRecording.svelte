<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy, onMount } from 'svelte';
  import {
    isRecording,
    currentSession,
    recordingStatus,
    formatTime,
    formatFileSize,
  } from '../stores';
  import ProcessingStages from './ProcessingStages.svelte';

  let isStopping = false;
  let isCancelling = false;
  let unlistenStatusUpdate;
  let processingStepStartTime = null;
  let previousStep = null;
  let completionTimeout = null;

  onMount(async () => {
    // Listen to file watcher status updates
    // The file watcher emits 'recording-status-update' events whenever the status JSON changes
    unlistenStatusUpdate = await listen('recording-status-update', (event) => {
      const status = event.payload;
      console.log('Received status update from watcher:', status);

      if (status) {
        recordingStatus.set(status);

        // Track step transitions for minimum display time
        if (status.status === 'processing' && status.step !== previousStep) {
          processingStepStartTime = Date.now();
          previousStep = status.step;
          console.log(`Processing step changed to: ${status.step}/${status.total_steps}`);
        }

        // If completed, ensure minimum display time before transitioning
        if (status.status === 'completed') {
          console.log('Recording completed, scheduling UI cleanup');
          const minDisplayTime = 800; // milliseconds
          const elapsed = processingStepStartTime
            ? Date.now() - processingStepStartTime
            : minDisplayTime;
          const remainingTime = Math.max(0, minDisplayTime - elapsed);

          // Clear any existing timeout
          if (completionTimeout) {
            clearTimeout(completionTimeout);
          }

          completionTimeout = setTimeout(() => {
            isRecording.set(false);
            currentSession.set(null);
            completionTimeout = null;
          }, remainingTime + 5000); // +5000 for the 5s completion display
        }
      }
    });
  });

  onDestroy(() => {
    // Clean up event listener
    if (unlistenStatusUpdate) {
      unlistenStatusUpdate();
    }
    // Clean up any pending timeouts
    if (completionTimeout) {
      clearTimeout(completionTimeout);
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

  async function cancelRecording() {
    if (!$isRecording) return;

    isCancelling = true;
    try {
      await invoke('cancel_recording', {
        sessionId: $currentSession,
      });

      console.log('Cancel signal sent, recording will be discarded...');
      // Clean up immediately since we're not waiting for processing
      isRecording.set(false);
      currentSession.set(null);
      recordingStatus.set(null);
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    } catch (error) {
      console.error('Failed to cancel recording:', error);
      // On error, clean up anyway
      isRecording.set(false);
      currentSession.set(null);
      recordingStatus.set(null);
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    } finally {
      isCancelling = false;
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
  <!-- Processing Status with Stages -->
  {#if $recordingStatus.status === 'processing'}
    <div class="processing-container">
      <!-- Header Badge -->
      <div class="processing-header-simple">
        <div class="badge">⏳ Processing</div>
        <span class="session-id-badge">{$recordingStatus.session_id || 'N/A'}</span>
      </div>

      <!-- Stages Component -->
      <ProcessingStages status={$recordingStatus} />
    </div>
  {:else if $recordingStatus.status === 'completed'}
    <!-- Completion Screen -->
    <div class="unified-processing">
      <!-- Header Badge -->
      <div class="processing-header-simple">
        <div class="badge completed">✓ Complete</div>
        <span class="session-id-badge">{$recordingStatus.session_id || 'N/A'}</span>
      </div>

      <!-- Main Content -->
      <div class="processing-content-simple">
        <!-- Checkmark -->
        <div class="status-icon">
          <svg class="checkmark" width="56" height="56" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
        </div>

        <!-- Status Message -->
        <div class="status-title">
          {$recordingStatus.message || 'Recording completed!'}
        </div>

        <!-- Metadata -->
        <div class="metadata-simple">
          {#if $recordingStatus.duration_secs}
            <div class="metadata-item-simple">
              <span class="label">Duration:</span>
              <span class="value">{formatTime($recordingStatus.duration_secs)}</span>
            </div>
          {/if}
          {#if $recordingStatus.file_size_mb}
            <div class="metadata-item-simple">
              <span class="label">Size:</span>
              <span class="value">{$recordingStatus.file_size_mb}</span>
            </div>
          {/if}
        </div>

        <!-- File Info -->
        {#if $recordingStatus.filename}
          <div class="file-info-simple">
            <div class="filename">{$recordingStatus.filename}</div>
            {#if $recordingStatus.file_path}
              <div class="filepath"><code>{$recordingStatus.file_path}</code></div>
            {/if}
          </div>
        {/if}
      </div>
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

  <!-- Action Buttons (only show during recording) -->
  {#if $recordingStatus.status === 'recording'}
  <div class="action-buttons">
    <button
      class="btn btn-secondary btn-lg cancel-btn"
      on:click={cancelRecording}
      disabled={isCancelling || isStopping}
    >
      {#if isCancelling}
        <svg class="spin" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <circle cx="12" cy="12" r="2"/>
        </svg>
        Cancelling...
      {:else}
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
        Cancel
      {/if}
    </button>
    <button
      class="btn btn-danger btn-lg stop-btn"
      on:click={stopRecording}
      disabled={isStopping || isCancelling}
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
  </div>
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

  /* Timer Card - Balanced & Readable */
  .timer-card {
    background: var(--bg-surface);
    border: 2px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xxl);
    margin-bottom: var(--spacing-lg);
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
    gap: var(--spacing-md);
    font-size: 48px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
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
    height: 6px;
    background: var(--bg-elevated);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: var(--spacing-sm);
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
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .channel-card {
    background: var(--gradient-surface);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 2px solid var(--border-subtle);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    min-height: 180px;
    display: flex;
    flex-direction: column;
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
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--spacing-md);
    box-shadow: var(--shadow-sm);
    flex-shrink: 0;
  }

  .channel-icon.system {
    background: linear-gradient(135deg, var(--accent-cyan) 0%, var(--accent-magenta) 100%);
    color: var(--text-on-accent);
  }

  .channel-icon.mic {
    background: linear-gradient(135deg, var(--accent-magenta) 0%, var(--rec-active) 100%);
    color: white;
  }

  .channel-content {
    flex: 1;
    display: flex;
    flex-direction: column;
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
    margin-bottom: var(--spacing-md);
  }

  .channel-stats svg {
    opacity: 0.5;
    flex-shrink: 0;
  }

  .frames-count {
    font-weight: 600;
    font-size: 16px;
    color: var(--accent-default);
    transition: all 0.3s ease;
    min-width: 70px;
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
    justify-content: flex-start;
    gap: 3px;
    height: 28px;
    margin-top: auto;
  }

  .bar {
    width: 3px;
    background: linear-gradient(180deg, var(--success), var(--success) 50%, transparent 50%);
    border-radius: 2px;
    animation: wave 1s ease-in-out infinite;
    animation-delay: var(--delay);
  }

  @keyframes wave {
    0%, 100% {
      height: 6px;
    }
    50% {
      height: 28px;
    }
  }

  /* Action Buttons */
  .action-buttons {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: var(--spacing-md);
    width: 100%;
  }

  .cancel-btn {
    background: linear-gradient(135deg, #6c757d 0%, #5a6268 100%);
    box-shadow: 0 4px 12px rgba(108, 117, 125, 0.3);
  }

  .cancel-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, #5a6268 0%, #495057 100%);
    box-shadow: 0 6px 20px rgba(108, 117, 125, 0.4);
  }

  .stop-btn {
    /* Existing stop button styles remain */
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

  /* Enhanced Processing UI - Prominent & Informative */
  .processing-header-v2 {
    position: relative;
    background: var(--gradient-surface);
    border: 3px solid var(--warning);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xxl);
    margin-bottom: var(--spacing-lg);
    overflow: hidden;
  }

  /* Pulsing Border Animation */
  .processing-pulse-border {
    position: absolute;
    inset: -3px;
    border: 3px solid var(--warning);
    border-radius: var(--radius-lg);
    opacity: 0.5;
    animation: borderPulse 2s ease-in-out infinite;
    pointer-events: none;
  }

  @keyframes borderPulse {
    0%, 100% {
      opacity: 0.5;
      transform: scale(1);
    }
    50% {
      opacity: 1;
      transform: scale(1.01);
    }
  }

  .processing-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-lg);
  }

  /* Step Indicator */
  .step-indicator {
    width: 100%;
    text-align: center;
  }

  .step-number {
    display: inline-block;
    padding: var(--spacing-sm) var(--spacing-xl);
    background: var(--warning-bg);
    color: var(--warning);
    font-size: 14px;
    font-weight: 800;
    letter-spacing: 0.1em;
    border-radius: var(--radius-md);
    border: 2px solid var(--warning);
  }

  /* Large Animated Spinner */
  .spinner-large {
    position: relative;
    width: 64px;
    height: 64px;
    margin: var(--spacing-md) 0;
  }

  .spinner-ring {
    position: absolute;
    inset: 0;
    border: 4px solid transparent;
    border-top-color: var(--warning);
    border-right-color: var(--warning);
    border-radius: 50%;
    animation: spinLarge 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
  }

  .spinner-ring-secondary {
    position: absolute;
    inset: 8px;
    border: 3px solid transparent;
    border-bottom-color: var(--accent-yellow);
    border-left-color: var(--accent-yellow);
    border-radius: 50%;
    animation: spinLarge 1.5s cubic-bezier(0.5, 0, 0.5, 1) infinite reverse;
    opacity: 0.6;
  }

  @keyframes spinLarge {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  /* Processing Title */
  .processing-title {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    text-align: center;
    line-height: 1.4;
  }

  /* File Metadata */
  .processing-metadata {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .metadata-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .metadata-item svg {
    opacity: 0.7;
    flex-shrink: 0;
  }

  .metadata-type {
    padding: 4px var(--spacing-sm);
    background: var(--warning-bg);
    color: var(--warning);
    border-radius: var(--radius-sm);
    font-weight: 600;
  }

  /* Progress Bar - Step Based */
  .processing-progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
  }

  .processing-progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 0 0 16px rgba(255, 184, 77, 0.4);
  }

  .processing-progress-fill-indeterminate {
    height: 100%;
    width: 40%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    position: absolute;
    animation: indeterminateProgress 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    box-shadow: 0 0 16px rgba(255, 184, 77, 0.4);
  }

  @keyframes indeterminateProgress {
    0% {
      left: -40%;
    }
    100% {
      left: 100%;
    }
  }

  .processing-progress-shine {
    position: absolute;
    inset: 0;
    background: linear-gradient(90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.3) 50%,
      transparent 100%
    );
    animation: progressShine 2s linear infinite;
  }

  @keyframes progressShine {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(200%);
    }
  }

  .progress-percentage {
    text-align: center;
    font-size: 14px;
    font-weight: 600;
    color: var(--warning);
    margin-top: var(--spacing-sm);
  }

  .processing-speed {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    font-family: 'IBM Plex Mono', monospace;
  }

  .session-id-small {
    font-size: 11px;
    color: var(--text-tertiary);
    font-family: 'Consolas', 'Monaco', monospace;
    opacity: 0.7;
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

  /* Processing Container */
  .processing-container {
    margin-bottom: var(--spacing-lg);
  }

  /* Unified Processing & Completion Screen */
  .unified-processing {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    margin-bottom: var(--spacing-lg);
    border: 2px solid var(--border-subtle);
  }

  .processing-header-simple {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
    padding-bottom: var(--spacing-lg);
    border-bottom: 1px solid var(--border-subtle);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: linear-gradient(135deg, var(--warning) 0%, var(--accent-yellow) 100%);
    color: white;
    font-weight: 700;
    font-size: 14px;
    border-radius: var(--radius-md);
    letter-spacing: 0.5px;
  }

  .badge.completed {
    background: linear-gradient(135deg, var(--success) 0%, #10B981 100%);
  }

  .session-id-badge {
    font-size: 12px;
    font-family: 'Consolas', 'Monaco', monospace;
    color: var(--text-secondary);
  }

  .processing-content-simple {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-lg);
    text-align: center;
  }

  .status-icon {
    margin: var(--spacing-md) 0;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--border-subtle);
    border-top-color: var(--warning);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .checkmark {
    color: var(--success);
    animation: popIn 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes popIn {
    0% { transform: scale(0.3); opacity: 0; }
    100% { transform: scale(1); opacity: 1; }
  }

  .status-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
    margin: var(--spacing-sm) 0;
  }

  .progress-container {
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 0 0 12px rgba(255, 184, 77, 0.3);
  }

  .progress-shine {
    position: absolute;
    inset: 0;
    background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.3) 50%, transparent 100%);
    animation: shine 2s linear infinite;
  }

  @keyframes shine {
    from { transform: translateX(-100%); }
    to { transform: translateX(200%); }
  }

  .progress-bar-indeterminate {
    width: 100%;
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
  }

  .progress-fill-indeterminate {
    height: 100%;
    width: 40%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    position: absolute;
    animation: indeterminateSlide 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    box-shadow: 0 0 12px rgba(255, 184, 77, 0.3);
  }

  @keyframes indeterminateSlide {
    0% { left: -40%; }
    100% { left: 100%; }
  }

  .progress-info {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 13px;
  }

  .progress-percent {
    font-weight: 700;
    color: var(--warning);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-speed {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .progress-label {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .metadata-simple {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    width: 100%;
    max-width: 400px;
    justify-content: center;
  }

  .metadata-item-simple {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: 13px;
  }

  .metadata-item-simple .label {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .metadata-item-simple .value {
    color: var(--text-primary);
    font-weight: 600;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .file-info-simple {
    width: 100%;
    max-width: 400px;
    padding: var(--spacing-md);
    background: var(--success-bg);
    border: 1px solid var(--success);
    border-radius: var(--radius-md);
  }

  .filename {
    font-weight: 600;
    color: var(--success);
    margin-bottom: var(--spacing-sm);
    word-break: break-word;
  }

  .filepath {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .filepath code {
    display: block;
    padding: var(--spacing-xs);
    background: rgba(0, 0, 0, 0.1);
    border-radius: var(--radius-sm);
    margin-top: var(--spacing-xs);
    font-family: 'Consolas', 'Monaco', monospace;
    word-break: break-all;
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
