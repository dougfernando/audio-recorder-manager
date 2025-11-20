<script>
  const componentStart = performance.now();

  import { onMount, afterUpdate, tick } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import RecordingPanel from './lib/components/RecordingPanel.svelte';
  import ActiveRecording from './lib/components/ActiveRecording.svelte';
  import RecordingsList from './lib/components/RecordingsList.svelte';
  import RecordingDetail from './lib/components/RecordingDetail.svelte';
  import Recovery from './lib/components/Recovery.svelte';
  import Settings from './lib/components/Settings.svelte';

  import {
    isRecording,
    currentSession,
    recordingStatus,
    recordings,
  } from './lib/stores';

  let activeTab = 'record';
  let hasLoadedRecordings = false;
  let firstRenderComplete = false;
  let selectedRecording = null; // Track which recording to show in detail view

  // Debug: log recordings data
  $: console.log('[App] recordings store:', $recordings);
  $: console.log('[App] selectedRecording:', selectedRecording);

  // Track first render completion
  afterUpdate(() => {
    if (!firstRenderComplete) {
      firstRenderComplete = true;
      console.log('[TIMING] First render complete (afterUpdate):', performance.now() - componentStart, 'ms');
    }
  });

  onMount(async () => {
    const mountStart = performance.now();

    // Listen for recording status updates from backend
    const unlisten = await listen('recording-status-update', (event) => {
      recordingStatus.set(event.payload);

      // Update isRecording based on status
      if (event.payload.status === 'recording') {
        isRecording.set(true);
        currentSession.set(event.payload.session_id);
      } else if (event.payload.status === 'completed' || event.payload.status === 'stopped') {
        isRecording.set(false);
        currentSession.set(null);
        // Reload recordings list
        loadRecordings();
      }
    });

    // Load recordings for sidebar on mount
    await loadRecordings();

    console.log('[TIMING] Component fully mounted and ready');

    // Cleanup on unmount
    return () => {
      unlisten();
    };
  });

  async function loadRecordings() {
    try {
      const result = await invoke('list_recordings');
      recordings.set(result);
      hasLoadedRecordings = true;
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }

  async function handleRecordingDeleted() {
    // First, reload the recordings to update the underlying data
    await loadRecordings();
    // Then, switch the view back to the main recordings list
    switchTab('recordings');
  }

  function handleRecordingRenamed(event) {
    const updatedRecording = event.detail;
    
    // Update the selected recording to the new one
    selectedRecording = updatedRecording;

    // Find and update the recording in the main list using the creation date as a key
    const updatedRecordings = $recordings.map(r => 
      r.created === updatedRecording.created ? updatedRecording : r
    );
    recordings.set(updatedRecordings);
  }

  function switchTab(tab, recording = null) {
    // If a recording is provided, show the detail view
    if (recording) {
      selectedRecording = recording;
      activeTab = 'recording-detail';
    } else {
      activeTab = tab;
      // Clear selected recording when switching away from detail view
      if (tab !== 'recording-detail') {
        selectedRecording = null;
      }
    }

    // Lazy load recordings when user first visits the recordings tab
    if (tab === 'recordings' && !hasLoadedRecordings) {
      loadRecordings();
    }
  }
</script>

<main>
  <!-- Sidebar -->
  <aside class="sidebar">
    <!-- Sidebar Header -->
    <div class="sidebar-header">
      <div class="sidebar-brand">
        <div class="brand-icon">
          <svg width="24" height="24" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="16" cy="16" r="14" fill="currentColor" opacity="0.8"/>
            <circle cx="16" cy="16" r="8" fill="white"/>
            <circle cx="16" cy="16" r="4" fill="currentColor"/>
          </svg>
        </div>
        <span class="brand-text">Audio Recorder Manager</span>
      </div>
      <button class="icon-btn" on:click={() => switchTab('settings')} title="Settings">
        <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
          <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319z"/>
        </svg>
      </button>
    </div>

    <!-- Record New Button -->
    <button class="record-new-btn" on:click={() => switchTab('record')}>
      <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
        <circle cx="8" cy="8" r="6"/>
      </svg>
      Record New
    </button>

    <!-- Recordings List -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">Recent Recordings</span>
        <button class="text-btn" on:click={() => switchTab('recordings')}>View All</button>
      </div>
      <div class="recordings-list-sidebar">
        {#if hasLoadedRecordings}
          {#if $recordings.length > 0}
            {#each $recordings.slice(0, 8) as recording (recording.path)}
              <button class="recording-item" on:click={() => switchTab('recordings', recording)}>
                <div class="recording-icon">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <circle cx="8" cy="8" r="3"/>
                  </svg>
                </div>
                <div class="recording-info">
                  <div class="recording-name">{recording.filename || 'Untitled'}</div>
                  <div class="recording-date">{new Date(recording.created).toLocaleDateString()}</div>
                </div>
              </button>
            {/each}
          {:else}
            <div class="empty-state">
              <svg width="32" height="32" viewBox="0 0 16 16" fill="currentColor" opacity="0.3">
                <circle cx="8" cy="8" r="6"/>
              </svg>
              <p>No recordings yet</p>
            </div>
          {/if}
        {:else}
          <div class="empty-state">
            <p>Click "Record New" to start</p>
          </div>
        {/if}
      </div>
    </div>

    <!-- Additional Navigation -->
    <div class="sidebar-footer">
      <button class="footer-btn {activeTab === 'recovery' ? 'active' : ''}" on:click={() => switchTab('recovery')}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 2a6 6 0 100 12A6 6 0 008 2zm0 1.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9z"/>
          <path d="M8 5v4l3 1.5"/>
        </svg>
        Recovery
      </button>
    </div>
  </aside>

  <!-- Main Content Area -->
  <div class="main-content">
    <div class="content fade-in">
      {#if activeTab === 'record'}
        <!-- State-aware Recording View -->
        {#if $isRecording}
          <!-- When Recording: Show only active recording (prominent, full-width) -->
          <div class="recording-active-view">
            <ActiveRecording />
          </div>
        {:else}
          <!-- When Idle: Show recording controls -->
          <div class="recording-idle-view">
            <RecordingPanel />
          </div>
        {/if}
      {:else if activeTab === 'recordings'}
        <RecordingsList onRecordingClick={(recording) => switchTab('recording-detail', recording)} />
      {:else if activeTab === 'recording-detail' && selectedRecording}
        <RecordingDetail
          recording={selectedRecording}
          onBack={() => switchTab('recordings')}
          on:deleted={handleRecordingDeleted}
          on:renamed={handleRecordingRenamed}
        />
      {:else if activeTab === 'recovery'}
        <Recovery />
      {:else if activeTab === 'settings'}
        <Settings />
      {/if}
    </div>
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: row;
    background: var(--bg-base);
    position: relative;
  }

  /* Sidebar - Asymmetric Command Panel */
  .sidebar {
    width: 280px;
    height: 100vh;
    background: var(--bg-elevated);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .sidebar::after {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    width: 1px;
    height: 100%;
    background: linear-gradient(180deg,
      transparent 0%,
      var(--accent-cyan) 20%,
      var(--accent-cyan) 80%,
      transparent 100%
    );
    opacity: 0.3;
  }

  /* Sidebar Header - Bold & Diagonal */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xl) var(--spacing-lg);
    border-bottom: 2px solid var(--border-subtle);
    position: relative;
    background: linear-gradient(135deg, rgba(0, 229, 255, 0.05) 0%, transparent 100%);
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .brand-icon {
    width: 32px;
    height: 32px;
    background: var(--gradient-primary);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-on-accent);
    box-shadow: var(--shadow-glow-cyan);
  }

  .brand-text {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    line-height: 1.2;
  }

  .icon-btn {
    width: 40px;
    height: 40px;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .icon-btn:hover {
    background: var(--bg-surface);
    color: var(--accent-cyan);
    border-color: var(--accent-cyan);
    transform: rotate(90deg);
  }

  /* Record New Button - Explosive CTA */
  .record-new-btn {
    margin: var(--spacing-lg);
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--gradient-recording);
    color: #FFFFFF;
    border-radius: var(--radius-sm);
    font-size: 14px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    box-shadow: var(--shadow-md), var(--shadow-glow-magenta);
    position: relative;
    overflow: hidden;
  }

  .record-new-btn::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.2), transparent);
    opacity: 0;
    transition: opacity 0.2s;
  }

  .record-new-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg), var(--shadow-glow-magenta);
  }

  .record-new-btn:hover:not(:disabled)::before {
    opacity: 1;
  }

  .record-new-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  /* Sidebar Section */
  .sidebar-section {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    margin-bottom: var(--spacing-sm);
  }

  .section-title {
    font-size: 10px;
    font-weight: 700;
    color: var(--text-accent);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .text-btn {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    background: transparent;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border: 1px solid transparent;
  }

  .text-btn:hover {
    color: var(--accent-cyan);
    background: var(--bg-surface);
    border-color: var(--accent-cyan);
  }

  /* Recordings List in Sidebar - Geometric Cards */
  .recordings-list-sidebar {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0 var(--spacing-sm);
  }

  .recording-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-bottom: var(--spacing-xs);
    background: var(--bg-surface);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    transition: all 0.2s ease;
    text-align: left;
    position: relative;
  }

  .recording-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--gradient-primary);
    opacity: 0;
    transition: opacity 0.2s;
  }

  .recording-item:hover {
    background: var(--bg-elevated);
    border-color: var(--accent-cyan);
    transform: translateX(4px);
  }

  .recording-item:hover::before {
    opacity: 1;
  }

  .recording-icon {
    width: 36px;
    height: 36px;
    background: linear-gradient(135deg, rgba(0, 229, 255, 0.1), rgba(255, 0, 110, 0.1));
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-cyan);
    flex-shrink: 0;
  }

  .recording-info {
    flex: 1;
    min-width: 0;
  }

  .recording-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    letter-spacing: -0.01em;
  }

  .recording-date {
    font-size: 10px;
    font-family: 'IBM Plex Mono', monospace;
    color: var(--text-tertiary);
    margin-top: 2px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxl) var(--spacing-lg);
    text-align: center;
    color: var(--text-tertiary);
  }

  .empty-state svg {
    margin-bottom: var(--spacing-md);
    opacity: 0.3;
  }

  .empty-state p {
    font-size: 12px;
    font-weight: 500;
  }

  /* Sidebar Footer - Minimal Accent */
  .sidebar-footer {
    padding: var(--spacing-lg);
    border-top: 2px solid var(--border-subtle);
    background: linear-gradient(180deg, transparent, rgba(0, 229, 255, 0.03));
  }

  .footer-btn {
    width: 100%;
    padding: var(--spacing-md);
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    font-size: 13px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .footer-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .footer-btn.active {
    background: linear-gradient(135deg, rgba(0, 229, 255, 0.15), rgba(255, 0, 110, 0.15));
    color: var(--accent-cyan);
    font-weight: 700;
    border-color: var(--accent-cyan);
  }

  /* Main Content Area - Spacious & Bold */
  .main-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .main-content::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg,
      var(--accent-cyan) 0%,
      transparent 20%,
      transparent 80%,
      var(--accent-magenta) 100%
    );
  }

  .content {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--spacing-xxxl);
  }

  /* Recording Active View - Full width, prominent */
  .recording-active-view {
    max-width: 1000px;
    margin: 0 auto;
    animation: slideInRight 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* Recording Idle View - Single column centered layout */
  .recording-idle-view {
    max-width: 700px;
    margin: 0 auto;
    animation: fadeIn 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* Responsive - Stack on smaller screens */
  @media (max-width: 1024px) {
    .recording-idle-view {
      grid-template-columns: 1fr;
      max-width: 700px;
    }

    .content {
      padding: var(--spacing-lg);
    }

    .sidebar {
      width: 220px;
    }
  }

  @media (max-width: 768px) {
    .sidebar {
      width: 180px;
    }

    .brand-text {
      display: none;
    }

    .section-title {
      font-size: 10px;
    }

    .recording-name {
      font-size: 12px;
    }
  }

  /* Smooth transitions */
  .fade-in {
    animation: fadeIn 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  }

</style>
