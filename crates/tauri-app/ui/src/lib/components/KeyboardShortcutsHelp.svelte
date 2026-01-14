<script>
  import { X } from 'lucide-svelte';
  import { formatShortcut, shortcuts } from '../keyboard.js';

  export let isOpen = false;

  function close() {
    isOpen = false;
  }

  function handleKeydown(event) {
    if (event.key === 'Escape') {
      close();
    }
  }

  // Group shortcuts by category
  const shortcutGroups = [
    {
      title: 'Gravação',
      shortcuts: [
        { name: 'START_RECORDING', label: 'Iniciar gravação' },
        { name: 'STOP_RECORDING', label: 'Parar gravação' }
      ]
    },
    {
      title: 'Transcrição',
      shortcuts: [
        { name: 'TRANSCRIBE', label: 'Iniciar transcrição' },
        { name: 'VIEW_TRANSCRIPT', label: 'Visualizar transcrição' },
        { name: 'COPY_MARKDOWN', label: 'Copiar markdown' }
      ]
    },
    {
      title: 'Navegação',
      shortcuts: [
        { name: 'PLAY_PAUSE', label: 'Play/Pause (no player)' },
        { name: 'ESCAPE', label: 'Fechar modal/cancelar' },
        { name: 'HELP', label: 'Mostrar/ocultar esta ajuda' }
      ]
    },
    {
      title: 'Aplicação',
      shortcuts: [
        { name: 'QUIT_APP', label: 'Sair da aplicação' }
      ]
    }
  ];
</script>

{#if isOpen}
  <div class="modal-overlay" on:click={close} on:keydown={handleKeydown}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h2>Atalhos de Teclado</h2>
        <button class="close-btn" on:click={close} aria-label="Fechar">
          <X size={20} />
        </button>
      </div>

      <div class="shortcuts-container">
        {#each shortcutGroups as group}
          <div class="shortcut-group">
            <h3>{group.title}</h3>
            <div class="shortcuts-list">
              {#each group.shortcuts as item}
                {@const shortcut = shortcuts[item.name]}
                {#if shortcut}
                  <div class="shortcut-item">
                    <span class="shortcut-label">{item.label}</span>
                    <kbd class="shortcut-keys">{formatShortcut(shortcut)}</kbd>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="modal-footer">
        <p class="hint">
          Pressione <kbd>?</kbd> ou <kbd>F1</kbd> a qualquer momento para ver estes atalhos
        </p>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-content {
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
    border-radius: 16px;
    padding: 0;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    background: linear-gradient(135deg, #5B9DFF 0%, #FF5C8D 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .close-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    padding: 8px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .shortcuts-container {
    padding: 24px;
    overflow-y: auto;
    max-height: calc(80vh - 180px);
  }

  .shortcut-group {
    margin-bottom: 32px;
  }

  .shortcut-group:last-child {
    margin-bottom: 0;
  }

  .shortcut-group h3 {
    margin: 0 0 16px 0;
    font-size: 14px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.2s;
  }

  .shortcut-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(91, 157, 255, 0.3);
  }

  .shortcut-label {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.9);
  }

  .shortcut-keys,
  kbd {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 12px;
    font-weight: 600;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.9);
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  .modal-footer {
    padding: 20px 24px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(0, 0, 0, 0.2);
  }

  .hint {
    margin: 0;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
    text-align: center;
  }

  .hint kbd {
    padding: 4px 8px;
    font-size: 11px;
  }

  /* Scrollbar styling */
  .shortcuts-container::-webkit-scrollbar {
    width: 8px;
  }

  .shortcuts-container::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
  }

  .shortcuts-container::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .shortcuts-container::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
