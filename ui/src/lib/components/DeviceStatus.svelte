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
      {#each $devices as device, index}
        <div class="device-item">
          <div class="device-icon {index === 0 ? 'primary' : 'secondary'}">
            {#if device.name.toLowerCase().includes('microphone') || device.name.toLowerCase().includes('mic')}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 14c1.66 0 2.99-1.34 2.99-3L15 5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.3-3c0 3-2.54 5.1-5.3 5.1S6.7 14 6.7 11H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c3.28-.48 6-3.3 6-6.72h-1.7z"/>
              </svg>
            {:else}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
              </svg>
            {/if}
          </div>
          <div class="device-info">
            <div class="device-name">{device.name}</div>
            <div class="device-specs">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                <circle cx="8" cy="8" r="2"/>
              </svg>
              {device.sample_rate / 1000}kHz
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                <circle cx="8" cy="8" r="2"/>
              </svg>
              {device.channels} ch
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
    margin-bottom: var(--spacing-lg);
  }

  .card-title {
    margin-bottom: 0;
  }

  .btn-sm {
    padding: 6px var(--spacing-md);
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
    gap: var(--spacing-md);
  }

  .device-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background-color: var(--card-background-secondary);
    border: 1px solid var(--stroke-surface);
    border-radius: var(--corner-radius-medium);
    transition: all 0.2s ease;
  }

  .device-item:hover {
    border-color: var(--stroke-surface-flyout);
    box-shadow: var(--elevation-card);
  }

  .device-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: var(--corner-radius-medium);
    color: white;
  }

  .device-icon.primary {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .device-icon.secondary {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  }

  .device-info {
    flex: 1;
  }

  .device-name {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .device-specs {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    font-family: 'Consolas', 'Monaco', monospace;
  }

  .device-specs svg {
    opacity: 0.4;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxxl) var(--spacing-lg);
    color: var(--text-tertiary);
  }

  .empty-state svg {
    margin-bottom: var(--spacing-md);
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
