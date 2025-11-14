<script>
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/tauri';
  import RecordingPanel from './lib/components/RecordingPanel.svelte';
  import ActiveRecording from './lib/components/ActiveRecording.svelte';
  import DeviceStatus from './lib/components/DeviceStatus.svelte';
  import RecordingsList from './lib/components/RecordingsList.svelte';
  import Recovery from './lib/components/Recovery.svelte';
  import {
    isRecording,
    currentSession,
    recordingStatus,
    recordings,
    devices,
  } from './lib/stores';

  let activeTab = 'record';

  onMount(async () => {
    // Load initial data
    await loadRecordings();
    await loadDevices();

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

    // Cleanup on unmount
    return () => {
      unlisten();
    };
  });

  async function loadRecordings() {
    try {
      const result = await invoke('list_recordings');
      recordings.set(result);
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }

  async function loadDevices() {
    try {
      const result = await invoke('get_status');
      if (result.devices) {
        devices.set(result.devices);
      }
    } catch (error) {
      console.error('Failed to load devices:', error);
    }
  }

  function switchTab(tab) {
    activeTab = tab;
  }
</script>

<main>
  <!-- Windows 11 Style Title Bar -->
  <div class="title-bar">
    <div class="title-content">
      <div class="app-icon">
        <svg width="20" height="20" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="16" cy="16" r="14" fill="currentColor" opacity="0.8"/>
          <circle cx="16" cy="16" r="8" fill="white"/>
          <circle cx="16" cy="16" r="4" fill="currentColor"/>
        </svg>
      </div>
      <h1 class="app-title">Audio Recorder Manager</h1>
    </div>
  </div>

  <!-- Windows 11 Style Tab Navigation -->
  <nav class="tab-nav">
    <button
      class="nav-tab {activeTab === 'record' ? 'active' : ''}"
      on:click={() => switchTab('record')}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <circle cx="8" cy="8" r="6"/>
      </svg>
      Record
    </button>
    <button
      class="nav-tab {activeTab === 'recordings' ? 'active' : ''}"
      on:click={() => switchTab('recordings')}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M2 3h12v2H2V3zm0 4h12v2H2V7zm0 4h12v2H2v-2z"/>
      </svg>
      Recordings
    </button>
    <button
      class="nav-tab {activeTab === 'recovery' ? 'active' : ''}"
      on:click={() => switchTab('recovery')}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 2a6 6 0 100 12A6 6 0 008 2zm0 1.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9z"/>
        <path d="M8 5v4l3 1.5"/>
      </svg>
      Recovery
    </button>
  </nav>

  <!-- Main Content Area -->
  <div class="content-wrapper">
    <div class="content fade-in">
      {#if activeTab === 'record'}
        <!-- Single Column Layout for Recording -->
        <div class="record-container">
          <RecordingPanel />
          <ActiveRecording />
          <DeviceStatus />
        </div>
      {:else if activeTab === 'recordings'}
        <RecordingsList />
      {:else if activeTab === 'recovery'}
        <Recovery />
      {/if}
    </div>
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--layer-fill-default);
  }

  /* Windows 11 Style Title Bar */
  .title-bar {
    background-color: var(--card-background);
    backdrop-filter: blur(40px);
    border-bottom: 1px solid var(--divider-stroke);
    padding: var(--spacing-md) var(--spacing-xxl);
    flex-shrink: 0;
  }

  .title-content {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .app-icon {
    width: 20px;
    height: 20px;
    color: var(--accent-default);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    line-height: 20px;
  }

  /* Windows 11 Style Navigation Tabs */
  .tab-nav {
    display: flex;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-xxl) 0 var(--spacing-xxl);
    background-color: var(--card-background);
    border-bottom: 1px solid var(--divider-stroke);
    flex-shrink: 0;
  }

  .nav-tab {
    position: relative;
    padding: var(--spacing-sm) var(--spacing-lg);
    background-color: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 400;
    border: none;
    border-radius: var(--corner-radius-small) var(--corner-radius-small) 0 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    transition: all 0.08s ease;
    min-height: 32px;
    border-bottom: 2px solid transparent;
  }

  .nav-tab:hover {
    background-color: var(--layer-fill-alt);
    color: var(--text-primary);
  }

  .nav-tab.active {
    background-color: var(--layer-fill-default);
    color: var(--accent-default);
    font-weight: 600;
    border-bottom-color: var(--accent-default);
  }

  .nav-tab svg {
    opacity: 0.7;
  }

  .nav-tab.active svg {
    opacity: 1;
  }

  /* Content Area */
  .content-wrapper {
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

  /* Single Column Layout - Windows 11 Style */
  .record-container {
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .title-bar,
    .tab-nav,
    .content {
      padding-left: var(--spacing-lg);
      padding-right: var(--spacing-lg);
    }

    .nav-tab {
      font-size: 12px;
      padding: var(--spacing-sm) var(--spacing-md);
    }

    .nav-tab svg {
      width: 14px;
      height: 14px;
    }

    .record-container {
      max-width: 100%;
    }
  }

  /* Smooth transitions between tabs */
  .fade-in {
    animation: fadeIn 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  }
</style>
