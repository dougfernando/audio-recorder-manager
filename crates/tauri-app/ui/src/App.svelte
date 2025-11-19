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
    background: var(--layer-fill-default);
  }

  /* Sidebar */
  .sidebar {
    width: 256px;
    height: 100vh;
    background: var(--layer-fill-alt);
    border-right: 1px solid var(--stroke-surface);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  /* Sidebar Header */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--stroke-surface);
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .brand-icon {
    width: 24px;
    height: 24px;
    color: var(--accent-default);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .brand-text {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--corner-radius-small);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.08s ease;
  }

  .icon-btn:hover {
    background: var(--card-background);
    color: var(--text-primary);
  }

  /* Record New Button */
  .record-new-btn {
    margin: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--accent-default);
    color: var(--text-on-accent);
    border-radius: var(--corner-radius-small);
    font-size: 14px;
    font-weight: 500;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    transition: all 0.08s ease;
  }

  .record-new-btn:hover:not(:disabled) {
    background: var(--accent-secondary);
  }

  .record-new-btn:active:not(:disabled) {
    background: var(--accent-tertiary);
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
    padding: var(--spacing-sm) var(--spacing-lg);
    margin-bottom: var(--spacing-sm);
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .text-btn {
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    padding: 4px 8px;
    border-radius: var(--corner-radius-small);
    transition: all 0.08s ease;
  }

  .text-btn:hover {
    color: var(--accent-default);
    background: var(--card-background);
  }

  /* Recordings List in Sidebar */
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
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-xs);
    background: transparent;
    color: var(--text-primary);
    border-radius: var(--corner-radius-small);
    transition: all 0.08s ease;
    text-align: left;
  }

  .recording-item:hover {
    background: var(--card-background);
  }

  .recording-icon {
    width: 32px;
    height: 32px;
    background: var(--card-background);
    border-radius: var(--corner-radius-small);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-default);
    flex-shrink: 0;
  }

  .recording-info {
    flex: 1;
    min-width: 0;
  }

  .recording-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recording-date {
    font-size: 11px;
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
  }

  .empty-state p {
    font-size: 13px;
  }

  /* Sidebar Footer */
  .sidebar-footer {
    padding: var(--spacing-lg);
    border-top: 1px solid var(--stroke-surface);
  }

  .footer-btn {
    width: 100%;
    padding: var(--spacing-md);
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--corner-radius-small);
    font-size: 14px;
    font-weight: 400;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    transition: all 0.08s ease;
  }

  .footer-btn:hover {
    background: var(--card-background);
    color: var(--text-primary);
  }

  .footer-btn.active {
    background: var(--card-background);
    color: var(--accent-default);
    font-weight: 500;
  }

  /* Main Content Area */
  .main-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .content {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--spacing-xxl);
  }

  /* Recording Active View - Full width, prominent */
  .recording-active-view {
    max-width: 900px;
    margin: 0 auto;
  }

  /* Recording Idle View - Single column centered layout */
  .recording-idle-view {
    max-width: 600px;
    margin: 0 auto;
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
