/**
 * Keyboard shortcuts management utility
 * Provides centralized keyboard shortcut handling with platform detection
 */

/**
 * Detect if running on macOS
 */
export const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;

/**
 * Get the modifier key name for the current platform
 */
export const modifierKey = isMac ? '⌘' : 'Ctrl';

/**
 * Keyboard shortcuts configuration
 */
export const shortcuts = {
  START_RECORDING: { key: 'r', ctrl: true, description: 'Iniciar gravação' },
  STOP_RECORDING: { key: 's', ctrl: true, description: 'Parar gravação' },
  TRANSCRIBE: { key: 't', ctrl: true, description: 'Iniciar transcrição' },
  VIEW_TRANSCRIPT: { key: 'e', ctrl: true, description: 'Visualizar transcrição' },
  COPY_MARKDOWN: { key: 'c', ctrl: true, description: 'Copiar markdown' },
  QUIT_APP: { key: 'q', ctrl: true, description: 'Sair da aplicação' },
  HELP: { key: '?', shift: true, description: 'Mostrar ajuda' },
  HELP_F1: { key: 'F1', description: 'Mostrar ajuda' },
  ESCAPE: { key: 'Escape', description: 'Fechar modal/cancelar' },
  PLAY_PAUSE: { key: ' ', description: 'Play/Pause' }
};

/**
 * Check if an event matches a keyboard shortcut
 * @param {KeyboardEvent} event - The keyboard event
 * @param {Object} shortcut - The shortcut configuration
 * @returns {boolean} - True if the event matches the shortcut
 */
export function matchesShortcut(event, shortcut) {
  const key = event.key.toLowerCase();
  const shortcutKey = shortcut.key.toLowerCase();

  // Check if key matches
  if (key !== shortcutKey) {
    return false;
  }

  // Check modifier keys
  const needsCtrl = shortcut.ctrl || false;
  const needsShift = shortcut.shift || false;
  const needsAlt = shortcut.alt || false;

  const hasCtrl = isMac ? event.metaKey : event.ctrlKey;
  const hasShift = event.shiftKey;
  const hasAlt = event.altKey;

  return (needsCtrl === hasCtrl || (!needsCtrl && !hasCtrl)) &&
         (needsShift === hasShift || (!needsShift && !hasShift)) &&
         (needsAlt === hasAlt || (!needsAlt && !hasAlt));
}

/**
 * Check if the current focus is on an input element
 * @returns {boolean} - True if an input element is focused
 */
export function isInputFocused() {
  const activeElement = document.activeElement;
  if (!activeElement) return false;

  const tagName = activeElement.tagName.toLowerCase();
  const isEditable = activeElement.isContentEditable;

  return (
    tagName === 'input' ||
    tagName === 'textarea' ||
    tagName === 'select' ||
    isEditable
  );
}

/**
 * Format a shortcut for display
 * @param {Object} shortcut - The shortcut configuration
 * @returns {string} - Formatted shortcut string
 */
export function formatShortcut(shortcut) {
  const parts = [];

  if (shortcut.ctrl) {
    parts.push(modifierKey);
  }
  if (shortcut.shift) {
    parts.push('Shift');
  }
  if (shortcut.alt) {
    parts.push('Alt');
  }

  // Format the key
  let keyDisplay = shortcut.key;
  if (keyDisplay === ' ') {
    keyDisplay = 'Space';
  } else if (keyDisplay.length === 1) {
    keyDisplay = keyDisplay.toUpperCase();
  }

  parts.push(keyDisplay);

  return parts.join('+');
}

/**
 * Create a keyboard shortcut handler
 * @param {Object} handlers - Map of shortcut names to handler functions
 * @param {Object} options - Options for the handler
 * @returns {Function} - Event handler function
 */
export function createShortcutHandler(handlers, options = {}) {
  const { ignoreInputs = true } = options;

  return function(event) {
    // Skip if an input is focused (unless override)
    if (ignoreInputs && isInputFocused()) {
      return;
    }

    // Check each shortcut
    for (const [name, handler] of Object.entries(handlers)) {
      const shortcut = shortcuts[name];
      if (!shortcut) continue;

      if (matchesShortcut(event, shortcut)) {
        event.preventDefault();
        event.stopPropagation();
        handler(event);
        return;
      }
    }
  };
}

/**
 * Get aria-keyshortcuts attribute value for an element
 * @param {string} shortcutName - The name of the shortcut
 * @returns {string} - The aria-keyshortcuts value
 */
export function getAriaKeyshortcuts(shortcutName) {
  const shortcut = shortcuts[shortcutName];
  if (!shortcut) return '';

  const parts = [];

  if (shortcut.ctrl) {
    parts.push(isMac ? 'Meta' : 'Control');
  }
  if (shortcut.shift) {
    parts.push('Shift');
  }
  if (shortcut.alt) {
    parts.push('Alt');
  }

  parts.push(shortcut.key);

  return parts.join('+');
}
