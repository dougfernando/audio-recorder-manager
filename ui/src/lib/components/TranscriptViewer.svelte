<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';
  import { marked } from 'marked';

  export let transcriptPath = '';
  export let recordingName = '';
  export let onClose = () => {};

  let transcriptContent = '';
  let renderedHtml = '';
  let isLoading = true;
  let errorMessage = '';
  let viewMode = 'rendered'; // 'rendered' or 'raw'

  onMount(async () => {
    await loadTranscript();
  });

  async function loadTranscript() {
    isLoading = true;
    errorMessage = '';
    try {
      transcriptContent = await invoke('read_transcript', { filePath: transcriptPath });
      renderedHtml = marked.parse(transcriptContent);
    } catch (error) {
      console.error('Failed to load transcript:', error);
      errorMessage = `Failed to load transcript: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  async function openInEditor() {
    try {
      await invoke('open_recording', { filePath: transcriptPath });
    } catch (error) {
      alert(`Failed to open transcript in editor: ${error}`);
    }
  }

  function downloadTranscript() {
    const blob = new Blob([transcriptContent], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = recordingName.replace(/\.(wav|m4a)$/i, '.md');
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function copyToClipboard() {
    navigator.clipboard.writeText(transcriptContent).then(() => {
      alert('Transcript copied to clipboard!');
    }).catch(err => {
      alert('Failed to copy transcript: ' + err);
    });
  }
</script>

<div class="transcript-viewer-overlay" on:click={onClose} on:keydown={(e) => e.key === 'Escape' && onClose()}>
  <div class="transcript-viewer" on:click|stopPropagation on:keydown|stopPropagation>
    <div class="viewer-header">
      <div class="header-info">
        <h2>Transcript</h2>
        <p class="recording-name">{recordingName}</p>
      </div>
      <div class="header-actions">
        <div class="view-mode-toggle">
          <button
            class="toggle-btn {viewMode === 'rendered' ? 'active' : ''}"
            on:click={() => viewMode = 'rendered'}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
            Preview
          </button>
          <button
            class="toggle-btn {viewMode === 'raw' ? 'active' : ''}"
            on:click={() => viewMode = 'raw'}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="16 18 22 12 16 6"/>
              <polyline points="8 6 2 12 8 18"/>
            </svg>
            Markdown
          </button>
        </div>
        <button class="btn btn-secondary btn-sm" on:click={openInEditor} title="Open in external editor">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </button>
        <button class="btn btn-secondary btn-sm" on:click={copyToClipboard} title="Copy to clipboard">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
        </button>
        <button class="btn btn-secondary btn-sm" on:click={downloadTranscript} title="Download">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
        </button>
        <button class="btn btn-secondary btn-sm close-btn" on:click={onClose}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="viewer-content">
      {#if isLoading}
        <div class="loading-state">
          <div class="spinner"></div>
          <p>Loading transcript...</p>
        </div>
      {:else if errorMessage}
        <div class="error-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <p>{errorMessage}</p>
          <button class="btn btn-primary" on:click={loadTranscript}>Try Again</button>
        </div>
      {:else if viewMode === 'rendered'}
        <div class="markdown-content">
          {@html renderedHtml}
        </div>
      {:else}
        <pre class="raw-content"><code>{transcriptContent}</code></pre>
      {/if}
    </div>
  </div>
</div>

<style>
  .transcript-viewer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--spacing-xl);
  }

  .transcript-viewer {
    background: white;
    border-radius: var(--corner-radius-large);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.2);
    max-width: 1200px;
    width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg) var(--spacing-xl);
    border-bottom: 1px solid var(--stroke-surface);
    background: linear-gradient(180deg, rgba(255, 255, 255, 1) 0%, rgba(249, 249, 249, 1) 100%);
  }

  .header-info h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .recording-name {
    margin: var(--spacing-xs) 0 0 0;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .header-actions {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .view-mode-toggle {
    display: flex;
    background: var(--card-background-secondary);
    border-radius: var(--corner-radius-small);
    padding: 2px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border-radius: var(--corner-radius-small);
    transition: all 0.15s ease;
  }

  .toggle-btn.active {
    background: white;
    color: var(--accent-default);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .toggle-btn:hover:not(.active) {
    color: var(--text-primary);
  }

  .viewer-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xl);
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 300px;
    gap: var(--spacing-lg);
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid rgba(0, 103, 192, 0.1);
    border-top-color: var(--accent-default);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-state {
    color: var(--text-secondary);
  }

  .error-state svg {
    color: var(--danger);
  }

  .markdown-content {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    line-height: 1.7;
    color: var(--text-primary);
  }

  .markdown-content :global(h1) {
    font-size: 28px;
    font-weight: 700;
    margin: var(--spacing-xl) 0 var(--spacing-lg) 0;
    color: var(--text-primary);
    border-bottom: 2px solid var(--stroke-surface);
    padding-bottom: var(--spacing-md);
  }

  .markdown-content :global(h2) {
    font-size: 22px;
    font-weight: 600;
    margin: var(--spacing-lg) 0 var(--spacing-md) 0;
    color: var(--text-primary);
  }

  .markdown-content :global(h3) {
    font-size: 18px;
    font-weight: 600;
    margin: var(--spacing-md) 0 var(--spacing-sm) 0;
    color: var(--text-secondary);
  }

  .markdown-content :global(p) {
    margin: var(--spacing-md) 0;
  }

  .markdown-content :global(ul), .markdown-content :global(ol) {
    margin: var(--spacing-md) 0;
    padding-left: var(--spacing-xl);
  }

  .markdown-content :global(li) {
    margin: var(--spacing-xs) 0;
  }

  .markdown-content :global(strong) {
    font-weight: 600;
    color: var(--text-primary);
  }

  .markdown-content :global(code) {
    background: var(--card-background-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
  }

  .markdown-content :global(pre) {
    background: var(--card-background-secondary);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    overflow-x: auto;
    margin: var(--spacing-md) 0;
  }

  .markdown-content :global(pre code) {
    background: transparent;
    padding: 0;
  }

  .raw-content {
    background: var(--card-background-secondary);
    padding: var(--spacing-lg);
    border-radius: var(--corner-radius-medium);
    overflow-x: auto;
    margin: 0;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
  }

  .raw-content code {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>
