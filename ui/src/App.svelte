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
  <header>
    <div class="header-content">
      <div class="logo">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="16" cy="16" r="14" fill="#4a90e2"/>
          <circle cx="16" cy="16" r="8" fill="white"/>
          <circle cx="16" cy="16" r="4" fill="#4a90e2"/>
        </svg>
        <h1>Audio Recorder Manager</h1>
      </div>
      <div class="tabs">
        <button
          class="tab {activeTab === 'record' ? 'active' : ''}"
          on:click={() => switchTab('record')}
        >
          Record
        </button>
        <button
          class="tab {activeTab === 'recordings' ? 'active' : ''}"
          on:click={() => switchTab('recordings')}
        >
          Recordings
        </button>
        <button
          class="tab {activeTab === 'recovery' ? 'active' : ''}"
          on:click={() => switchTab('recovery')}
        >
          Recovery
        </button>
      </div>
    </div>
  </header>

  <div class="content">
    {#if activeTab === 'record'}
      <div class="record-view">
        <div class="left-panel">
          <RecordingPanel />
          <DeviceStatus />
        </div>
        <div class="right-panel">
          <ActiveRecording />
        </div>
      </div>
    {:else if activeTab === 'recordings'}
      <RecordingsList />
    {:else if activeTab === 'recovery'}
      <Recovery />
    {/if}
  </div>
</main>

<style>
  main {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
  }

  header {
    background-color: var(--bg-primary);
    border-bottom: 1px solid var(--border-color);
    box-shadow: var(--shadow-sm);
  }

  .header-content {
    padding: 16px 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .tabs {
    display: flex;
    gap: 8px;
  }

  .tab {
    padding: 8px 20px;
    background-color: transparent;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    border-radius: var(--radius-md);
    transition: all 0.2s ease;
  }

  .tab:hover {
    background-color: var(--bg-secondary);
  }

  .tab.active {
    background-color: var(--primary-color);
    color: white;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .record-view {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
    height: 100%;
  }

  .left-panel,
  .right-panel {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  @media (max-width: 1024px) {
    .record-view {
      grid-template-columns: 1fr;
    }
  }
</style>
