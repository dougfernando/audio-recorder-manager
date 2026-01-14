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
    padding: 2px 4px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 3px;
    font-size: 9px;
    font-weight: 500;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: rgba(255, 255, 255, 0.6);
    letter-spacing: 0.3px;
    margin-left: 6px;
    white-space: nowrap;
    text-transform: uppercase;
  }

  .keyboard-shortcut-inline {
    font-size: 10px;
    font-weight: 500;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: rgba(255, 255, 255, 0.6);
    letter-spacing: 0.3px;
  }
</style>
