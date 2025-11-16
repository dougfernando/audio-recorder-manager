<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Transcription settings
  let apiKey = '';
  let model = 'gemini-2.5-flash';
  let prompt = '';
  let optimizeAudio = false;

  // Storage paths
  let recordingsDir = '';
  let transcriptionsDir = '';

  let isSaving = false;
  let isLoading = true;
  let saveMessage = '';
  let errorMessage = '';

  const defaultPrompt = `Please process the attached audio file and provide the following two sections in markdown format:

**1. Raw Transcription:**

*   Detect the language spoken in the audio.
*   Transcribe the audio verbatim in the detected language, word for word, exactly as spoken.
*   Use appropriate punctuation.
*   Indicate long pauses with [...].
*   If there are multiple speakers, label them as "Speaker 1:", "Speaker 2:", etc.

**2. Key Topics Discussed:**

*   Analyze the raw transcription.
*   Identify the main subjects, decisions, and action items.
*   Organize these points into a summary with clear headings for each topic.
*   Describe the key topics in the same language as identified in the raw transcription as long it is Spanish, Portuguese or English; otherwise, use English.
*   Ensure no critical information is lost.

Your entire response should be a single markdown document.`;

  onMount(async () => {
    await loadConfig();
  });

  async function loadConfig() {
    isLoading = true;
    errorMessage = '';
    try {
      // Load transcription config
      const transcriptionConfig = await invoke('load_transcription_config');
      apiKey = transcriptionConfig.api_key || '';
      model = transcriptionConfig.model || 'gemini-2.5-flash';
      prompt = transcriptionConfig.prompt || defaultPrompt;
      optimizeAudio = transcriptionConfig.optimize_audio || false;

      // Load recorder config (storage paths)
      const recorderConfig = await invoke('load_recorder_config');
      recordingsDir = recorderConfig.recordings_dir || 'storage/recordings';
      transcriptionsDir = recorderConfig.transcriptions_dir || 'storage/transcriptions';
    } catch (error) {
      console.error('Failed to load config:', error);
      errorMessage = `Failed to load configuration: ${error}`;
      // Set defaults
      prompt = defaultPrompt;
      recordingsDir = 'storage/recordings';
      transcriptionsDir = 'storage/transcriptions';
    } finally {
      isLoading = false;
    }
  }

  async function saveConfig() {
    isSaving = true;
    saveMessage = '';
    errorMessage = '';

    try {
      // Save transcription config
      await invoke('save_transcription_config', {
        config: {
          api_key: apiKey,
          model: model,
          prompt: prompt,
          optimize_audio: optimizeAudio,
        },
      });

      // Save recorder config (storage paths)
      await invoke('save_recorder_config', {
        recordingsDir: recordingsDir,
        transcriptionsDir: transcriptionsDir,
      });

      saveMessage = 'Settings saved successfully!';
      setTimeout(() => {
        saveMessage = '';
      }, 3000);
    } catch (error) {
      console.error('Failed to save config:', error);
      errorMessage = `Failed to save settings: ${error}`;
    } finally {
      isSaving = false;
    }
  }

  async function pickRecordingsFolder() {
    try {
      const result = await invoke('pick_folder', {
        defaultPath: recordingsDir || null,
      });
      if (result) {
        recordingsDir = result;
      }
    } catch (error) {
      console.error('Failed to pick folder:', error);
      errorMessage = `Failed to select folder: ${error}`;
    }
  }

  async function pickTranscriptionsFolder() {
    try {
      const result = await invoke('pick_folder', {
        defaultPath: transcriptionsDir || null,
      });
      if (result) {
        transcriptionsDir = result;
      }
    } catch (error) {
      console.error('Failed to pick folder:', error);
      errorMessage = `Failed to select folder: ${error}`;
    }
  }

  function resetPrompt() {
    prompt = defaultPrompt;
  }
</script>

<div class="settings-container">
  <div class="card settings-card">
    <h2 class="card-title">Transcription Settings</h2>

    {#if errorMessage}
      <div class="error-message">
        {errorMessage}
      </div>
    {/if}

    {#if saveMessage}
      <div class="success-message">
        {saveMessage}
      </div>
    {/if}

    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading settings...</p>
      </div>
    {:else}
      <form on:submit|preventDefault={saveConfig}>
        <h3 class="section-title">Storage Paths</h3>

        <div class="form-group">
          <label class="form-label" for="recordings-dir">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            Recordings Folder
          </label>
          <div class="path-input-group">
            <input
              id="recordings-dir"
              type="text"
              class="form-input"
              bind:value={recordingsDir}
              placeholder="storage/recordings"
              required
            />
            <button type="button" class="btn btn-secondary btn-sm" on:click={pickRecordingsFolder}>
              Browse
            </button>
          </div>
          <small class="form-hint">
            Location where audio recordings will be saved
          </small>
        </div>

        <div class="form-group">
          <label class="form-label" for="transcriptions-dir">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            Transcriptions Folder
          </label>
          <div class="path-input-group">
            <input
              id="transcriptions-dir"
              type="text"
              class="form-input"
              bind:value={transcriptionsDir}
              placeholder="storage/transcriptions"
              required
            />
            <button type="button" class="btn btn-secondary btn-sm" on:click={pickTranscriptionsFolder}>
              Browse
            </button>
          </div>
          <small class="form-hint">
            Location where transcription files will be saved
          </small>
        </div>

        <hr class="section-divider" />

        <h3 class="section-title">API Configuration</h3>

        <div class="form-group">
          <label class="form-label" for="api-key">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
            </svg>
            Google Gemini API Key
          </label>
          <input
            id="api-key"
            type="password"
            class="form-input"
            bind:value={apiKey}
            placeholder="Enter your Gemini API key"
            required
          />
          <small class="form-hint">
            Get your API key from <a href="https://aistudio.google.com/app/apikey" target="_blank">Google AI Studio</a>
          </small>
        </div>

        <div class="form-group">
          <label class="form-label" for="model">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
            Model
          </label>
          <select id="model" class="form-select" bind:value={model}>
            <option value="gemini-2.5-flash">Gemini 2.5 Flash (Recommended)</option>
            <option value="gemini-2.5-pro">Gemini 2.5 Pro</option>
            <option value="gemini-2.0-flash">Gemini 2.0 Flash</option>
            <option value="gemini-2.0-flash-exp">Gemini 2.0 Flash (Experimental)</option>
          </select>
          <small class="form-hint">Flash models are faster and cheaper for transcription</small>
        </div>

        <div class="form-group">
          <label class="form-label">
            <input type="checkbox" bind:checked={optimizeAudio} />
            Optimize audio before uploading
          </label>
          <small class="form-hint">
            Converts to mono 16kHz WAV to reduce file size (requires ffmpeg)
          </small>
        </div>

        <div class="form-group">
          <div class="prompt-label-row">
            <label class="form-label" for="prompt">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
                <line x1="10" y1="9" x2="8" y2="9"/>
              </svg>
              Transcription Prompt
            </label>
            <button type="button" class="btn btn-secondary btn-sm" on:click={resetPrompt}>
              Reset to Default
            </button>
          </div>
          <textarea
            id="prompt"
            class="form-textarea"
            bind:value={prompt}
            rows="12"
            required
          ></textarea>
          <small class="form-hint">
            Customize the instructions for how the AI should transcribe your audio
          </small>
        </div>

        <div class="button-row">
          <button type="submit" class="btn btn-primary btn-lg" disabled={isSaving}>
            {#if isSaving}
              Saving...
            {:else}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                <polyline points="17 21 17 13 7 13 7 21"/>
                <polyline points="7 3 7 8 15 8"/>
              </svg>
              Save Settings
            {/if}
          </button>
        </div>
      </form>
    {/if}
  </div>
</div>

<style>
  .settings-container {
    max-width: 900px;
    margin: 0 auto;
  }

  .settings-card {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(255, 255, 255, 0.85) 100%);
    border: 1px solid rgba(0, 103, 192, 0.1);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08), 0 2px 6px rgba(0, 103, 192, 0.05);
  }

  .error-message {
    background-color: var(--danger-bg);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
    font-size: 14px;
  }

  .success-message {
    background-color: var(--success-bg);
    border: 1px solid var(--success);
    color: var(--success);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-lg);
    font-size: 14px;
    font-weight: 500;
  }

  .loading-state {
    text-align: center;
    padding: var(--spacing-xxxl);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 4px solid rgba(0, 103, 192, 0.1);
    border-top-color: var(--accent-default);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin: 0 auto var(--spacing-lg);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .form-hint {
    display: block;
    margin-top: var(--spacing-xs);
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .form-hint a {
    color: var(--accent-default);
    text-decoration: none;
  }

  .form-hint a:hover {
    text-decoration: underline;
  }

  .form-textarea {
    width: 100%;
    padding: var(--spacing-md);
    border: 1px solid var(--stroke-surface);
    border-radius: var(--corner-radius-small);
    font-size: 13px;
    font-family: 'Consolas', 'Monaco', monospace;
    background-color: var(--card-background-secondary);
    color: var(--text-primary);
    transition: all 0.08s ease;
    resize: vertical;
    line-height: 1.6;
  }

  .form-textarea:hover {
    border-color: var(--stroke-surface-flyout);
  }

  .form-textarea:focus {
    border-color: var(--accent-default);
    box-shadow: 0 0 0 1px var(--accent-default);
    outline: none;
  }

  .prompt-label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: 13px;
    min-height: 28px;
  }

  .button-row {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--spacing-xl);
  }

  .form-label input[type="checkbox"] {
    margin-right: var(--spacing-sm);
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-lg);
    margin-top: var(--spacing-xl);
  }

  .section-title:first-child {
    margin-top: 0;
  }

  .section-divider {
    border: none;
    border-top: 1px solid var(--stroke-surface);
    margin: var(--spacing-xl) 0;
  }

  .path-input-group {
    display: flex;
    gap: var(--spacing-sm);
  }

  .path-input-group .form-input {
    flex: 1;
  }

  .path-input-group .btn {
    flex-shrink: 0;
  }
</style>
