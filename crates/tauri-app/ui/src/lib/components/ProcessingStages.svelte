<script>
  import { formatTime } from '../stores';

  export let status = null;

  // Define stages based on format
  $: isM4a = status?.format === 'm4a' || status?.filename?.endsWith('.m4a');
  $: currentStep = status?.step || 0;
  $: totalSteps = status?.total_steps || (isM4a ? 4 : 3);
  $: processingType = status?.processing_type || '';

  // Stage definitions
  $: stages = isM4a
    ? [
        {
          id: 1,
          name: 'Analyzing Audio',
          icon: '🔍',
          type: 'analyzing',
        },
        {
          id: 2,
          name: 'Merging Channels',
          icon: '🔀',
          type: 'merging',
        },
        {
          id: 3,
          name: 'Encoding to M4A',
          icon: '📦',
          type: 'encoding',
        },
        {
          id: 4,
          name: 'Finalizing',
          icon: '✓',
          type: 'finalizing',
        },
      ]
    : [
        {
          id: 1,
          name: 'Analyzing Audio',
          icon: '🔍',
          type: 'analyzing',
        },
        {
          id: 2,
          name: 'Merging Channels',
          icon: '🔀',
          type: 'merging',
        },
        {
          id: 3,
          name: 'Encoding',
          icon: '📦',
          type: 'encoding',
        },
      ];

  // Determine stage state
  function getStageState(stage) {
    if (stage.id < currentStep) return 'completed';
    if (stage.id === currentStep) return 'current';
    return 'pending';
  }

  // Get stage message - takes both stage and state to ensure reactivity
  function getStageMessage(stage, state) {
    if (state === 'pending') return 'Waiting...';

    if (state === 'completed') {
      // Special handling for analyzing stage
      if (stage.type === 'analyzing') {
        return status?.message || 'Detected system audio and microphone';
      }
      // All other completed stages show "Completed"
      return 'Completed';
    }

    // Current stage - show backend message or fallback
    if (state === 'current') {
      if (status?.message) {
        return status.message;
      }
      // Fallback messages
      switch (stage.type) {
        case 'analyzing':
          return 'Detecting audio channels...';
        case 'merging':
          return 'Combining audio streams...';
        case 'encoding':
          return isM4a ? 'Converting to M4A format...' : 'Processing audio...';
        case 'finalizing':
          return 'Saving recording...';
        default:
          return 'Processing...';
      }
    }

    return '';
  }

  // Show progress for current stage - only when actively processing (not at 0% or 100%)
  $: showProgress = currentStep > 0
    && status?.ffmpeg_progress !== null
    && status?.ffmpeg_progress !== undefined
    && status.ffmpeg_progress > 0
    && status.ffmpeg_progress < 100;
  $: progressPercent = status?.ffmpeg_progress ?? 0;
  $: processingSpeed = status?.processing_speed ?? null;

  // Enhanced progress tracking - ETA display
  // Use nullish coalescing (??) instead of OR (||) to preserve valid 0 values
  $: estimatedRemainingSecs = status?.estimated_remaining_secs ?? null;
  $: audioDurationMs = status?.audio_duration_ms ?? null;
  $: processedTimeMs = status?.processed_time_ms ?? null;

  // Debug logging for ETA fields (can be removed after confirming feature works)
  // $: if (status?.ffmpeg_progress !== undefined) {
  //   console.log('ProcessingStages ETA data:', {
  //     ffmpeg_progress: status?.ffmpeg_progress,
  //     estimated_remaining_secs: status?.estimated_remaining_secs,
  //     audio_duration_ms: status?.audio_duration_ms,
  //     processed_time_ms: status?.processed_time_ms,
  //     processing_speed: status?.processing_speed
  //   });
  // }

  // Format remaining time as "X min Y sec" or "X sec"
  function formatRemainingTime(secs) {
    if (secs === null || secs === undefined) return null;
    const minutes = Math.floor(secs / 60);
    const seconds = secs % 60;
    if (minutes > 0) {
      return `${minutes} min ${seconds} sec`;
    }
    return `${seconds} sec`;
  }

  // Format milliseconds as "MM:SS"
  function formatMsAsTime(ms) {
    if (ms === null || ms === undefined) return null;
    const totalSecs = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSecs / 60);
    const seconds = totalSecs % 60;
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  // Defensive: Clamp processed time to not exceed total duration (in case of bad backend data)
  $: clampedProcessedTimeMs = (processedTimeMs !== null && audioDurationMs !== null)
    ? Math.min(processedTimeMs, audioDurationMs)
    : processedTimeMs;

  $: remainingTimeDisplay = formatRemainingTime(estimatedRemainingSecs);
  $: processedTimeDisplay = formatMsAsTime(clampedProcessedTimeMs);
  $: totalTimeDisplay = formatMsAsTime(audioDurationMs);
</script>

<div class="processing-stages">
  <!-- Compact Header with Duration -->
  <div class="stages-header">
    <div class="header-left">
      <span class="stages-title">Stage {currentStep} of {totalSteps}</span>
      {#if (status?.duration_secs && status.duration_secs < 86400) || status?.audio_duration_ms}
        <span class="duration-inline">
          {#if (status?.duration_secs && status.duration_secs < 86400)}
            {formatTime(status.duration_secs)}
          {:else if status?.audio_duration_ms}
            {formatTime(Math.floor(status.audio_duration_ms / 1000))}
          {/if}
        </span>
      {/if}
    </div>
    <div class="progress-label-inline">{Math.round((currentStep / totalSteps) * 100)}%</div>
  </div>

  <!-- Compact Progress Bar -->
  <div class="progress-bar">
    <div class="progress-fill" style="width: {(currentStep / totalSteps) * 100}%">
      <div class="progress-shine"></div>
    </div>
  </div>

  <!-- Stages List -->
  <div class="stages-list">
    {#each stages as stage (stage.id)}
      {@const state = getStageState(stage)}
      <div class="stage-item" class:completed={state === 'completed'} class:current={state === 'current'} class:pending={state === 'pending'}>
        <!-- Stage Icon/Indicator -->
        <div class="stage-indicator">
          {#if state === 'completed'}
            <div class="stage-icon completed">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
            </div>
          {:else if state === 'current'}
            <div class="stage-icon current">
              <div class="spinner"></div>
            </div>
          {:else}
            <div class="stage-icon pending">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <circle cx="12" cy="12" r="8"/>
              </svg>
            </div>
          {/if}
          {#if stage.id < totalSteps}
            <div class="stage-connector" class:active={state === 'completed'}></div>
          {/if}
        </div>

        <!-- Stage Content -->
        <div class="stage-content">
          <span class="stage-name">{stage.icon} {stage.name}</span>
          {#if state === 'current'}
            <span class="stage-badge current">IN PROGRESS</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Compact Progress Section -->
  {#if (processingType === 'merging' || processingType === 'encoding')}
    <div class="progress-section-compact">
      {#if showProgress}
        <div class="progress-bar-large">
          <div class="progress-fill-large" style="width: {progressPercent}%">
            <div class="progress-shine"></div>
          </div>
        </div>

        <div class="progress-info">
          <div class="progress-main-compact">
            <span class="progress-percent-compact">{progressPercent}%</span>
            {#if processingSpeed}
              <span class="progress-speed-compact">{processingSpeed}</span>
            {/if}
          </div>

          {#if processedTimeDisplay && totalTimeDisplay && remainingTimeDisplay}
            <div class="progress-times">
              <span>{processedTimeDisplay} / {totalTimeDisplay}</span>
              <span class="remaining">~{remainingTimeDisplay}</span>
            </div>
          {/if}
        </div>
      {:else}
        <div class="progress-bar-large">
          <div class="progress-fill-indeterminate-large">
            <div class="progress-shine"></div>
          </div>
        </div>
        <div class="progress-preparing">Preparing...</div>
      {/if}
    </div>
  {/if}

  <!-- File Size (if available) -->
  {#if status?.file_size_mb}
    <div class="file-size-compact">
      Est. Size: {status.file_size_mb}
    </div>
  {/if}
</div>

<style>
  .processing-stages {
    background: var(--bg-surface);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 2px solid var(--border-subtle);
  }

  .stages-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .stages-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .duration-inline {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: 'IBM Plex Mono', monospace;
    padding: 2px 6px;
    background: var(--bg-elevated);
    border-radius: 4px;
  }

  .progress-label-inline {
    font-size: 12px;
    font-weight: 700;
    color: var(--warning);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    background: var(--bg-elevated);
    border-radius: 3px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
    margin-bottom: var(--spacing-sm);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    transition: width 0.4s ease;
    position: relative;
    overflow: hidden;
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

  /* Stages List */
  .stages-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .stage-item {
    display: flex;
    gap: var(--spacing-sm);
    position: relative;
    padding: 4px 0;
    transition: all 0.2s ease;
  }

  .stage-item.current {
    background: linear-gradient(90deg, transparent 0%, rgba(255, 184, 77, 0.05) 50%, transparent 100%);
    border-radius: var(--radius-sm);
    padding: 6px var(--spacing-sm);
    margin: 0 calc(-1 * var(--spacing-sm));
  }

  /* Stage Indicator */
  .stage-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex-shrink: 0;
  }

  .stage-icon {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    z-index: 1;
    transition: all 0.2s ease;
  }

  .stage-icon svg {
    width: 12px;
    height: 12px;
  }

  .stage-icon.completed {
    background: var(--success);
    color: white;
    box-shadow: 0 0 0 3px var(--success-bg);
  }

  .stage-icon.current {
    background: var(--warning);
    color: white;
    box-shadow: 0 0 0 3px rgba(255, 184, 77, 0.2);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      box-shadow: 0 0 0 3px rgba(255, 184, 77, 0.2);
    }
    50% {
      box-shadow: 0 0 0 4px rgba(255, 184, 77, 0.15);
    }
  }

  .stage-icon.pending {
    background: var(--bg-elevated);
    color: var(--text-tertiary);
    border: 1px solid var(--border-subtle);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .stage-connector {
    width: 2px;
    flex: 1;
    background: var(--border-subtle);
    margin-top: 2px;
    transition: background 0.2s ease;
  }

  .stage-connector.active {
    background: var(--success);
  }

  /* Stage Content */
  .stage-content {
    flex: 1;
    min-width: 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .stage-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stage-item.pending .stage-name {
    color: var(--text-tertiary);
  }

  .stage-badge {
    font-size: 9px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 8px;
    letter-spacing: 0.4px;
  }

  .stage-badge.current {
    background: rgba(255, 184, 77, 0.2);
    color: var(--warning);
  }

  /* Compact Progress Section */
  .progress-section-compact {
    background: var(--bg-surface);
    border: 2px solid var(--warning);
    border-radius: var(--radius-md);
    padding: var(--spacing-sm);
    margin-top: var(--spacing-sm);
    box-shadow: 0 0 16px rgba(255, 184, 77, 0.15);
  }

  .progress-bar-large {
    width: 100%;
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
    margin-bottom: var(--spacing-xs);
  }

  .progress-fill-large {
    height: 100%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    transition: width 0.3s ease;
    position: relative;
    overflow: hidden;
  }

  .progress-fill-indeterminate-large {
    height: 100%;
    width: 40%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    position: absolute;
    animation: indeterminateSlide 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  @keyframes indeterminateSlide {
    0% { left: -40%; }
    100% { left: 100%; }
  }

  .progress-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .progress-main-compact {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
  }

  .progress-percent-compact {
    font-size: 18px;
    font-weight: 700;
    color: var(--warning);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-speed-compact {
    font-size: 12px;
    color: var(--text-secondary);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-times {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-times .remaining {
    color: var(--warning);
    font-weight: 600;
  }

  .progress-preparing {
    text-align: center;
    font-size: 12px;
    color: var(--text-secondary);
    font-style: italic;
  }

  /* File Size Compact */
  .file-size-compact {
    text-align: center;
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: var(--spacing-xs);
    padding: 4px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
  }

  /* Responsive */
  @media (max-width: 480px) {
    .processing-stages {
      padding: var(--spacing-sm);
    }

    .stages-title {
      font-size: 11px;
    }

    .stage-icon {
      width: 20px;
      height: 20px;
    }

    .stage-icon svg {
      width: 10px;
      height: 10px;
    }

    .stage-name {
      font-size: 12px;
    }

    .progress-percent-compact {
      font-size: 16px;
    }

    .progress-speed-compact {
      font-size: 11px;
    }
  }
</style>
