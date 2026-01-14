<script>
  import { formatShortcut, shortcuts } from '../keyboard.js';

  /**
   * The name of the shortcut to display
   * @type {string}
   */
  export let shortcut = '';

  /**
   * Display style: 'badge' or 'inline'
   * @type {'badge' | 'inline'}
   */
  export let style = 'badge';

  $: shortcutConfig = shortcuts[shortcut];
  $: displayText = shortcutConfig ? formatShortcut(shortcutConfig) : '';
</script>

{#if displayText}
  {#if style === 'badge'}
    <span class="keyboard-shortcut-badge">
      {displayText}
    </span>
  {:else}
    <span class="keyboard-shortcut-inline">
      {displayText}
    </span>
  {/if}
{/if}

<style>
  .keyboard-shortcut-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: rgba(255, 255, 255, 0.8);
    letter-spacing: 0.5px;
    margin-left: 8px;
    white-space: nowrap;
  }

  .keyboard-shortcut-inline {
    font-size: 12px;
    font-weight: 600;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: rgba(255, 255, 255, 0.7);
    letter-spacing: 0.5px;
  }
</style>
