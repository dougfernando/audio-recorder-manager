# Performance Logging Guide

This document describes the comprehensive logging added to identify bottlenecks causing the black screen delay when loading the Tauri app.

## Overview

Timing logs have been added throughout the entire application stack to measure the time taken at each stage of initialization:

1. **Tauri Backend (Rust)** - Logs written to file
2. **HTML/DOM** - Browser console logs
3. **JavaScript Entry Point** - Browser console logs
4. **Svelte Components** - Browser console logs

## Log Locations

### Backend Logs (Tauri Rust)

**File Location**: `tauri_app.log` (in the same directory as the executable)

**What's logged**:
- Application start timestamp
- Tauri builder creation
- Plugin initialization (shell, dialog)
- App state setup
- Setup handler execution
- Directory creation
- File watcher setup
- Invoke handler configuration
- Application run loop start

**Format**: `[TIMING] <event>: <duration>`

**Example**:
```
[TIMING] App start: 0ms
[TIMING] Creating Tauri builder: 2ms
[TIMING] Initializing shell plugin: 5ms
[TIMING] Setup handler executing: 15ms
```

### Frontend Logs (Browser Console)

**Access**: Open Developer Tools (F12) → Console tab

#### 1. HTML Page Load (`index.html`)

**What's logged**:
- HTML parsing start
- DOMContentLoaded event
- Window load event
- Body parsing
- Script module loading

**Example**:
```
[TIMING] HTML parsing started
[TIMING] Body parsing started: 12ms from navigation
[TIMING] About to load main.js module: 15ms from navigation
[TIMING] DOMContentLoaded event fired: 45ms from navigation
```

#### 2. Main JavaScript Entry (`main.js`)

**What's logged**:
- Script load time
- CSS import time
- App.svelte import time
- App target element retrieval
- App instance creation time

**Example**:
```
[TIMING] main.js script loaded: 50ms from page load
[TIMING] Importing app.css: 0ms since script start
[TIMING] app.css imported: 5ms since script start
[TIMING] Importing App.svelte: 5ms since script start
[TIMING] App.svelte imported: 250ms since script start
[TIMING] App instance created: 275ms since script start
```

#### 3. App Component (`App.svelte`)

**What's logged**:
- Component script execution start
- Import statements (Svelte lifecycle, Tauri APIs, child components, stores)
- Component script setup completion
- onMount lifecycle start and completion
- Event listener setup
- First render completion (afterUpdate)
- First frame paint (requestAnimationFrame)

**Example**:
```
[TIMING] App.svelte script starting to execute
[TIMING] Imported lifecycle functions: 2ms
[TIMING] Imported @tauri-apps/api/event: 15ms
[TIMING] All components imported: 180ms
[TIMING] Stores imported: 185ms
[TIMING] App.svelte script setup complete: 190ms
[TIMING] App.svelte onMount started: 195ms
[TIMING] Event listener setup complete: 5ms
[TIMING] App.svelte onMount complete: 200ms
[TIMING] First render complete (afterUpdate): 210ms
[TIMING] First frame painted: 215ms
[TIMING] === UI IS NOW VISIBLE ===
```

## How to Use This Information

### Running the App

1. **Start the app** in development mode:
   ```bash
   cd crates/tauri-app
   npm run tauri dev
   ```

2. **Open Developer Tools** (F12) before or immediately after the app window appears

3. **Check both log sources**:
   - Browser Console: Press F12, go to Console tab
   - Backend Log File: Open `tauri_app.log` in the executable directory

### Interpreting the Logs

#### Identifying Bottlenecks

Look for large time gaps between consecutive log entries:

- **Backend (Rust)**:
  - If there's a large gap early (0-50ms range), the Tauri initialization is slow
  - If the gap is in setup handler (50-100ms range), directory creation or file watcher is slow

- **Frontend (Browser)**:
  - If `App.svelte imported` takes >500ms, component compilation is slow
  - If `All components imported` takes >300ms, child component loading is slow
  - If `First render complete` happens long after `script setup complete`, rendering is slow
  - If `First frame painted` is much later than `onMount complete`, there's a paint bottleneck

#### Common Bottleneck Scenarios

1. **Tauri/Rust Initialization Slow** (0-100ms):
   - Plugin initialization taking too long
   - Directory I/O operations slow

2. **JavaScript Module Loading Slow** (100-500ms):
   - Large bundle sizes
   - Many component imports
   - Slow module resolution

3. **Component Rendering Slow** (500ms+):
   - Heavy initial component rendering
   - Large CSS processing
   - Complex reactive computations

4. **Paint/Display Slow**:
   - CSS animations or transitions delaying visibility
   - GPU/rendering pipeline issues
   - Window manager composition delays

### Timeline Overview

A typical healthy startup should look like:

```
0ms     - Tauri app starts
50ms    - HTML loaded
100ms   - JavaScript loaded
300ms   - Components imported
400ms   - First render
450ms   - UI visible
```

If the UI is visible much later (2-3 seconds), identify which phase takes the longest.

## Next Steps

After collecting the logs:

1. **Identify the slowest phase** by comparing timestamps
2. **Focus optimization efforts** on that specific area
3. **Possible optimizations**:
   - Lazy load components
   - Reduce bundle size
   - Optimize CSS (remove unused styles)
   - Use code splitting
   - Preload critical resources
   - Optimize backend initialization

## Removing the Logs

Once bottlenecks are identified and fixed, you can remove or reduce logging:

- **Backend**: Comment out or remove `log::info!` calls in `main.rs`
- **Frontend**: Remove `console.log` calls from `index.html`, `main.js`, and `App.svelte`
