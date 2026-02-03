<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Transcription settings
  let apiKey = '';
  let model = 'gemini-2.5-flash';
  let prompt = '';
  let optimizeAudio = false;

  // Storage paths
  let storageDir = '';

  let isSaving = false;
  let isLoading = true;
  let saveMessage = '';
  let errorMessage = '';

  // Dynamic models
  let availableModels = [];
  let isLoadingModels = false;
  let modelsError = '';

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
      storageDir = recorderConfig.storage_dir || 'storage';
    } catch (error) {
      console.error('Failed to load config:', error);
      errorMessage = `Failed to load configuration: ${error}`;
      // Set defaults
      prompt = defaultPrompt;
      storageDir = 'storage';
    } finally {
      isLoading = false;
      // Load available models after config is loaded
      if (apiKey) {
        previousApiKey = apiKey;
        loadModels();
      }
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
        storageDir: storageDir,
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

  async function pickStorageFolder() {
    try {
      const result = await invoke('pick_folder', {
        defaultPath: storageDir || null,
      });
      if (result) {
        storageDir = result;
      }
    } catch (error) {
      console.error('Failed to pick folder:', error);
      errorMessage = `Failed to select folder: ${error}`;
    }
  }

  function resetPrompt() {
    prompt = defaultPrompt;
  }

  async function loadModels() {
    if (!apiKey) {
      availableModels = [];
      modelsError = '';
      return;
    }

    isLoadingModels = true;
    modelsError = '';

    try {
      const models = await invoke('list_gemini_models', { apiKey });
      availableModels = models;

      // If current model is not in the list, keep it as an option
      const modelIds = models.map(m => m.id);
      if (model && !modelIds.includes(model)) {
        availableModels = [{ id: model, display_name: model + ' (current)', description: '' }, ...models];
      }
    } catch (error) {
      console.error('Failed to load models:', error);
      modelsError = `Failed to load models: ${error}`;
      availableModels = [];
    } finally {
      isLoadingModels = false;
    }
  }

  // Load models when API key changes
  let previousApiKey = '';
  $: if (apiKey !== previousApiKey && !isLoading) {
    previousApiKey = apiKey;
    loadModels();
  }
</script>

<div class="settings-container">
  <div class="card settings-card">
    <div class="settings-header">
      <h2 class="card-title">Transcription Settings</h2>
    </div>

    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading settings...</p>
      </div>
    {:else}
      <form on:submit|preventDefault={saveConfig} class="settings-form">
        <div class="settings-content">
          <h3 class="section-title">Storage Path</h3>

        <div class="form-group">
          <label class="form-label" for="storage-dir">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            Storage Folder
          </label>
          <div class="path-input-group">
            <input
              id="storage-dir"
              type="text"
              class="form-input"
              bind:value={storageDir}
              placeholder="storage"
              required
            />
            <button type="button" class="btn btn-secondary btn-sm" on:click={pickStorageFolder}>
              Browse
            </button>
          </div>
          <small class="form-hint">
            Base location for recordings, transcriptions, and status files.
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
          <div class="model-label-row">
            <label class="form-label" for="model">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
                <line x1="8" y1="21" x2="16" y2="21"/>
                <line x1="12" y1="17" x2="12" y2="21"/>
              </svg>
              Model
            </label>
            {#if apiKey}
              <button type="button" class="btn btn-secondary btn-sm" on:click={loadModels} disabled={isLoadingModels}>
                {#if isLoadingModels}
                  <div class="btn-spinner-small"></div>
                {:else}
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
                  </svg>
                {/if}
                Refresh Models
              </button>
            {/if}
          </div>
          <select id="model" class="form-select" bind:value={model}>
            {#if availableModels.length > 0}
              {#each availableModels as m}
                <option value={m.id}>{m.display_name}</option>
              {/each}
            {:else}
              <option value="gemini-2.5-flash">Gemini 2.5 Flash</option>
              <option value="gemini-2.5-pro">Gemini 2.5 Pro</option>
              <option value="gemini-2.0-flash">Gemini 2.0 Flash</option>
            {/if}
          </select>
          {#if isLoadingModels}
            <small class="form-hint loading-hint">
              <div class="spinner-tiny"></div>
              Loading available models...
            </small>
          {:else if modelsError}
            <small class="form-hint error-hint">{modelsError}</small>
          {:else if availableModels.length > 0}
            <small class="form-hint">{availableModels.length} models available. Flash models are faster and cheaper for transcription.</small>
          {:else}
            <small class="form-hint">Enter your API key to load available models dynamically.</small>
          {/if}
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
        </div>

        <div class="settings-footer">
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

          <div class="button-row">
            <button type="submit" class="btn btn-primary btn-lg" disabled={isSaving}>
              {#if isSaving}
                <div class="btn-spinner"></div>
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
        </div>
      </form>
    {/if}
  </div>
</div>

<style>
  .settings-container {
    max-width: 900px;
    margin: 0 auto;
    padding: var(--spacing-lg);
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .settings-card {
    background: var(--card-background);
    border: 1px solid var(--stroke-surface);
    box-shadow: var(--elevation-card);
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    margin-bottom: 0;
  }

  .settings-header {
    padding: var(--spacing-lg) var(--spacing-lg) 0;
    flex-shrink: 0;
  }

  .settings-form {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
    padding-top: 0;
  }

  .settings-footer {
    flex-shrink: 0;
    padding: var(--spacing-lg);
    border-top: 1px solid var(--stroke-surface);
    background: var(--layer-fill-alt);
  }

  .error-message {
    background-color: var(--danger-bg);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-md);
    font-size: 14px;
    animation: slideDown 0.2s ease-out;
  }

  .success-message {
    background-color: var(--success-bg);
    border: 1px solid var(--success);
    color: var(--success);
    padding: var(--spacing-md);
    border-radius: var(--corner-radius-medium);
    margin-bottom: var(--spacing-md);
    font-size: 14px;
    font-weight: 500;
    animation: slideDown 0.2s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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

  .model-label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .btn-spinner-small {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .spinner-tiny {
    width: 10px;
    height: 10px;
    border: 2px solid rgba(0, 103, 192, 0.2);
    border-top-color: var(--accent-default);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    display: inline-block;
    vertical-align: middle;
  }

  .loading-hint {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .error-hint {
    color: var(--danger) !important;
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
    gap: var(--spacing-md);
  }

  .btn-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
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
    padding-left: var(--spacing-sm);
    border-left: 3px solid;
    border-image: var(--gradient-purple) 1;
  }

  .section-title:first-child {
    margin-top: 0;
    border-image: var(--gradient-blue) 1;
  }

  .section-title:nth-child(2) {
    border-image: var(--gradient-teal) 1;
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

  /* Responsive Design */
  @media (max-width: 768px) {
    .settings-container {
      padding: var(--spacing-md);
    }

    .settings-header,
    .settings-content,
    .settings-footer {
      padding: var(--spacing-md);
    }

    .card-title {
      font-size: 16px;
    }

    .section-title {
      font-size: 15px;
    }

    .path-input-group {
      flex-direction: column;
    }

    .path-input-group .btn {
      width: 100%;
    }

    .prompt-label-row {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }

    .button-row {
      justify-content: stretch;
    }

    .button-row .btn {
      flex: 1;
    }

    .form-textarea {
      font-size: 12px;
    }
  }

  @media (max-width: 480px) {
    .settings-container {
      padding: var(--spacing-sm);
    }

    .settings-header,
    .settings-content,
    .settings-footer {
      padding: var(--spacing-sm);
    }

    .card-title {
      font-size: 15px;
    }

    .section-title {
      font-size: 14px;
      margin-bottom: var(--spacing-md);
    }

    .form-group {
      margin-bottom: var(--spacing-md);
    }

    .form-label {
      font-size: 12px;
    }

    .form-input,
    .form-select {
      font-size: 13px;
    }

    .btn-sm {
      font-size: 12px;
      padding: var(--spacing-xs) var(--spacing-sm);
    }

    .btn-lg {
      font-size: 14px;
      padding: var(--spacing-sm) var(--spacing-lg);
    }
  }

  /* Dark mode adjustments */
  @media (prefers-color-scheme: dark) {
    .settings-card {
      background:
        var(--bg-gradient-blue),
        linear-gradient(135deg, rgba(42, 42, 42, 0.95) 0%, rgba(36, 36, 36, 0.85) 100%);
      border: 1px solid rgba(79, 172, 254, 0.3);
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.28), 0 2px 6px rgba(79, 172, 254, 0.15);
    }

    .settings-footer {
      background: linear-gradient(180deg, rgba(42, 42, 42, 0) 0%, rgba(32, 32, 32, 0.5) 100%);
    }
  }
</style>
