<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { devices } from '../stores';

  let isRefreshing = false;

  async function refreshDevices() {
    isRefreshing = true;
    try {
      const result = await invoke('get_status');
      if (result.devices) {
        devices.set(result.devices);
      }
    } catch (error) {
      console.error('Failed to refresh devices:', error);
    } finally {
      isRefreshing = false;
    }
  }
</script>

<div class="card">
  <div class="header">
    <h2 class="card-title">Audio Devices</h2>
    <button
      class="btn btn-secondary btn-sm"
      on:click={refreshDevices}
      disabled={isRefreshing}
    >
      {#if isRefreshing}
        <svg class="spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
        Refreshing...
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
        </svg>
        Refresh
      {/if}
    </button>
  </div>

  {#if $devices && $devices.length > 0}
    <div class="devices-list">
      {#each $devices as device}
        <div class="device-item">
          <div class="device-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
          </div>
          <div class="device-info">
            <div class="device-name">{device.name}</div>
            <div class="device-specs">
              {device.sample_rate / 1000}kHz • {device.channels} channels
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
        <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
        <line x1="12" y1="19" x2="12" y2="23"/>
        <line x1="8" y1="23" x2="16" y2="23"/>
      </svg>
      <p>No devices detected</p>
      <small>Click refresh to scan for audio devices</small>
    </div>
  {/if}
</div>

<style>
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .card-title {
    margin-bottom: 0;
  }

  .btn-sm {
    padding: 6px 12px;
    font-size: 13px;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .devices-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .device-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background-color: var(--bg-secondary);
    border-radius: var(--radius-md);
    transition: all 0.2s ease;
  }

  .device-item:hover {
    background-color: var(--bg-tertiary);
  }

  .device-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background-color: var(--bg-primary);
    border-radius: var(--radius-md);
    color: var(--primary-color);
  }

  .device-info {
    flex: 1;
  }

  .device-name {
    font-weight: 500;
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .device-specs {
    font-size: 12px;
    color: var(--text-secondary);
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    color: var(--text-tertiary);
  }

  .empty-state svg {
    margin-bottom: 12px;
    opacity: 0.3;
  }

  .empty-state p {
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .empty-state small {
    font-size: 12px;
  }
</style>
