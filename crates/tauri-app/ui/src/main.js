// Performance timing starts when script loads
const scriptStart = performance.now();
console.log('[TIMING] main.js script loaded:', scriptStart, 'ms from page load');

console.log('[TIMING] Importing app.css:', performance.now() - scriptStart, 'ms since script start');
import './app.css'

console.log('[TIMING] app.css imported:', performance.now() - scriptStart, 'ms since script start');
console.log('[TIMING] Importing App.svelte:', performance.now() - scriptStart, 'ms since script start');
import App from './App.svelte'

console.log('[TIMING] App.svelte imported:', performance.now() - scriptStart, 'ms since script start');

const appTarget = document.getElementById('app');
console.log('[TIMING] Got app target element:', performance.now() - scriptStart, 'ms since script start');

console.log('[TIMING] Creating App instance:', performance.now() - scriptStart, 'ms since script start');
const app = new App({
  target: appTarget,
})

console.log('[TIMING] App instance created:', performance.now() - scriptStart, 'ms since script start');
console.log('[TIMING] Total frontend initialization time:', performance.now() - scriptStart, 'ms');

// Make timing available globally for debugging
window.__appTiming = {
  scriptStart,
  appCreated: performance.now()
};

export default app
