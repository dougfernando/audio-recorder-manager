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
          name: 'Finalizing',
          icon: '✓',
          type: 'finalizing',
        },
      ];

  // Determine stage state
  function getStageState(stage) {
    if (stage.id < currentStep) return 'completed';
    if (stage.id === currentStep) return 'current';
    return 'pending';
  }

  // Get stage message
  function getStageMessage(stage) {
    const state = getStageState(stage);

    if (state === 'pending') return 'Waiting...';
    if (state === 'completed') {
      // Show completion messages
      if (stage.type === 'analyzing') {
        return status?.message || 'Audio detected';
      }
      return 'Completed';
    }

    // Current stage - show live message from backend
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
          return 'Converting to M4A format...';
        case 'finalizing':
          return 'Saving recording...';
        default:
          return 'Processing...';
      }
    }

    return '';
  }

  // Show progress for current stage
  $: showProgress = currentStep > 0 && status?.ffmpeg_progress !== null && status?.ffmpeg_progress !== undefined;
  $: progressPercent = status?.ffmpeg_progress || 0;
  $: processingSpeed = status?.processing_speed || null;

  // Enhanced progress tracking - ETA display
  $: estimatedRemainingSecs = status?.estimated_remaining_secs || null;
  $: audioDurationMs = status?.audio_duration_ms || null;
  $: processedTimeMs = status?.processed_time_ms || null;

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

  $: remainingTimeDisplay = formatRemainingTime(estimatedRemainingSecs);
  $: processedTimeDisplay = formatMsAsTime(processedTimeMs);
  $: totalTimeDisplay = formatMsAsTime(audioDurationMs);
</script>

<div class="processing-stages">
  <!-- Header -->
  <div class="stages-header">
    <span class="stages-title">Processing Stage {currentStep} of {totalSteps}</span>
  </div>

  <!-- Progress Bar (overall) -->
  <div class="overall-progress">
    <div class="progress-bar">
      <div class="progress-fill" style="width: {(currentStep / totalSteps) * 100}%">
        <div class="progress-shine"></div>
      </div>
    </div>
    <div class="progress-label">{Math.round((currentStep / totalSteps) * 100)}%</div>
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
          <div class="stage-header">
            <span class="stage-name">{stage.icon} {stage.name}</span>
            {#if state === 'current'}
              <span class="stage-badge current">IN PROGRESS</span>
            {/if}
          </div>

          <div class="stage-message">{getStageMessage(stage)}</div>

          <!-- FFmpeg Progress for current stage -->
          {#if state === 'current' && (stage.type === 'merging' || stage.type === 'encoding')}
            <div class="stage-progress">
              {#if showProgress}
                <!-- Determinate progress bar -->
                <div class="progress-bar-small">
                  <div class="progress-fill-small" style="width: {progressPercent}%">
                    <div class="progress-shine"></div>
                  </div>
                </div>
                <div class="progress-details">
                  <span class="progress-percent">{progressPercent}%</span>
                  {#if processingSpeed}
                    <span class="progress-speed">{processingSpeed}</span>
                  {/if}
                </div>
                <!-- Enhanced ETA display -->
                {#if remainingTimeDisplay || (processedTimeDisplay && totalTimeDisplay)}
                  <div class="progress-eta">
                    {#if processedTimeDisplay && totalTimeDisplay}
                      <span class="eta-processed">Processed: {processedTimeDisplay} / {totalTimeDisplay}</span>
                    {/if}
                    {#if remainingTimeDisplay}
                      <span class="eta-remaining">Est. remaining: {remainingTimeDisplay}</span>
                    {/if}
                  </div>
                {/if}
              {:else}
                <!-- Indeterminate progress bar (preparing) -->
                <div class="progress-bar-small">
                  <div class="progress-fill-indeterminate">
                    <div class="progress-shine"></div>
                  </div>
                </div>
                <div class="progress-details">
                  <span class="progress-label">Preparing...</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Metadata -->
  {#if status?.duration_secs || status?.file_size_mb}
    <div class="metadata">
      {#if status.duration_secs}
        <div class="metadata-item">
          <span class="label">Duration:</span>
          <span class="value">{formatTime(status.duration_secs)}</span>
        </div>
      {/if}
      {#if status.file_size_mb}
        <div class="metadata-item">
          <span class="label">Estimated Size:</span>
          <span class="value">{status.file_size_mb}</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .processing-stages {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    border: 2px solid var(--border-subtle);
  }

  .stages-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  .stages-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* Overall Progress */
  .overall-progress {
    margin-bottom: var(--spacing-xl);
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
    margin-bottom: var(--spacing-sm);
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

  .progress-label {
    text-align: center;
    font-size: 13px;
    font-weight: 600;
    color: var(--warning);
    font-family: 'IBM Plex Mono', monospace;
  }

  /* Stages List */
  .stages-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .stage-item {
    display: flex;
    gap: var(--spacing-md);
    position: relative;
    padding: var(--spacing-md) 0;
    transition: all 0.3s ease;
  }

  .stage-item.current {
    background: linear-gradient(90deg, transparent 0%, rgba(255, 184, 77, 0.05) 50%, transparent 100%);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin: 0 calc(-1 * var(--spacing-md));
  }

  /* Stage Indicator */
  .stage-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex-shrink: 0;
    padding-top: 2px;
  }

  .stage-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    z-index: 1;
    transition: all 0.3s ease;
  }

  .stage-icon.completed {
    background: var(--success);
    color: white;
    box-shadow: 0 0 0 4px var(--success-bg);
  }

  .stage-icon.current {
    background: var(--warning);
    color: white;
    box-shadow: 0 0 0 4px rgba(255, 184, 77, 0.2), 0 0 16px rgba(255, 184, 77, 0.3);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      box-shadow: 0 0 0 4px rgba(255, 184, 77, 0.2), 0 0 16px rgba(255, 184, 77, 0.3);
    }
    50% {
      box-shadow: 0 0 0 6px rgba(255, 184, 77, 0.1), 0 0 20px rgba(255, 184, 77, 0.4);
    }
  }

  .stage-icon.pending {
    background: var(--bg-elevated);
    color: var(--text-tertiary);
    border: 2px solid var(--border-subtle);
  }

  .spinner {
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

  .stage-connector {
    width: 2px;
    flex: 1;
    background: var(--border-subtle);
    margin-top: 4px;
    transition: background 0.3s ease;
  }

  .stage-connector.active {
    background: var(--success);
  }

  /* Stage Content */
  .stage-content {
    flex: 1;
    min-width: 0;
  }

  .stage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .stage-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stage-item.pending .stage-name {
    color: var(--text-tertiary);
  }

  .stage-badge {
    font-size: 10px;
    font-weight: 700;
    padding: 3px var(--spacing-sm);
    border-radius: 10px;
    letter-spacing: 0.5px;
  }

  .stage-badge.current {
    background: rgba(255, 184, 77, 0.2);
    color: var(--warning);
  }

  .stage-message {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-sm);
  }

  .stage-item.pending .stage-message {
    color: var(--text-tertiary);
    font-style: italic;
  }

  /* Stage Progress */
  .stage-progress {
    margin-top: var(--spacing-sm);
  }

  .progress-bar-small {
    width: 100%;
    height: 6px;
    background: var(--bg-elevated);
    border-radius: 3px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    position: relative;
    margin-bottom: var(--spacing-xs);
  }

  .progress-fill-small {
    height: 100%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    transition: width 0.3s ease;
    position: relative;
    overflow: hidden;
  }

  .progress-fill-indeterminate {
    height: 100%;
    width: 40%;
    background: linear-gradient(90deg, var(--warning) 0%, var(--accent-yellow) 100%);
    position: absolute;
    animation: indeterminateSlide 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    box-shadow: 0 0 12px rgba(255, 184, 77, 0.3);
  }

  @keyframes indeterminateSlide {
    0% { left: -40%; }
    100% { left: 100%; }
  }

  .progress-details {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 12px;
  }

  .progress-percent {
    font-weight: 700;
    color: var(--warning);
    font-family: 'IBM Plex Mono', monospace;
  }

  .progress-speed {
    color: var(--text-secondary);
  }

  .progress-label {
    color: var(--text-secondary);
    font-size: 12px;
    font-style: italic;
  }

  /* Enhanced ETA Display */
  .progress-eta {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .eta-processed {
    font-size: 12px;
    color: var(--text-secondary);
    font-family: 'IBM Plex Mono', monospace;
  }

  .eta-remaining {
    font-size: 13px;
    font-weight: 600;
    color: var(--warning);
  }

  /* Metadata */
  .metadata {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    margin-top: var(--spacing-lg);
    justify-content: center;
  }

  .metadata-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: 13px;
  }

  .metadata-item .label {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .metadata-item .value {
    color: var(--text-primary);
    font-weight: 600;
    font-family: 'Consolas', 'Monaco', monospace;
  }

  /* Responsive */
  @media (max-width: 480px) {
    .processing-stages {
      padding: var(--spacing-md);
    }

    .stage-item.current {
      margin: 0 calc(-1 * var(--spacing-sm));
      padding: var(--spacing-sm);
    }

    .stage-icon {
      width: 28px;
      height: 28px;
    }

    .stage-name {
      font-size: 14px;
    }

    .metadata {
      flex-direction: column;
      align-items: stretch;
    }

    .metadata-item {
      flex-direction: row;
      justify-content: space-between;
    }
  }
</style>
