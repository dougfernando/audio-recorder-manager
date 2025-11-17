# WebView2 Black Screen Fix

> **⚠️ UPDATE**: This approach was superseded after timing analysis revealed the actual bottleneck. See [WEBVIEW2_INITIALIZATION_DELAY.md](./WEBVIEW2_INITIALIZATION_DELAY.md) for the real fix.
>
> **TL;DR**: The delay was in WebView2 initialization (15 seconds), not UI rendering. Hiding the window didn't help because the delay happened during window creation itself.

---

## Original Approach (Not Used)

This document explains the original fix attempted to eliminate the black screen delay when loading the Tauri app on Windows.

## The Problem

When launching the Tauri app, users experienced:
1. **Window appears immediately** - The native window frame shows up
2. **Black screen for 2-3 seconds** - WebView2 content is not yet rendered
3. **UI finally appears** - After the delay, Svelte components become visible

This is a common issue with Tauri/WebView2 applications where the native window is displayed before the WebView2 content has finished initializing and rendering.

## Root Cause

The issue occurs because:
1. **Tauri creates and shows the window immediately** on startup
2. **WebView2 initialization takes time** (loading the Edge runtime, parsing HTML, executing JavaScript)
3. **Svelte component rendering takes time** (module loading, component initialization, first paint)
4. **The window is visible during steps 2-3**, showing a black/blank screen

## The Solution

We implemented a **deferred window visibility** pattern:

### 1. Window Configuration (`tauri.conf.json`)

```json
{
  "windows": [{
    "visible": false,        // Start hidden
    "backgroundColor": "#F5F5F7",  // Match app background
    "center": true,
    "transparent": false
  }]
}
```

**Key settings:**
- `"visible": false` - Window starts hidden
- `"backgroundColor": "#F5F5F7"` - Matches the app's gradient background color (prevents white flash)
- `"center": true` - Window appears centered when shown
- `"transparent": false` - Better performance than transparent windows

### 2. Backend Event Listener (`main.rs`)

```rust
// Show window when frontend is ready (avoids black screen)
let window = app.get_webview_window("main").expect("Failed to get main window");
let window_clone = window.clone();

// Listen for window-ready event from frontend
let _listener = window.listen("window-ready", move |_event| {
    log::info!("[TIMING] Received window-ready event");
    log::info!("[TIMING] Showing window now...");
    if let Err(e) = window_clone.show() {
        log::error!("Failed to show window: {}", e);
    } else {
        log::info!("[TIMING] Window shown successfully");
    }
});

// Fallback: Show window after 3 seconds if frontend doesn't emit ready event
let window_fallback = window.clone();
std::thread::spawn(move || {
    std::thread::sleep(std::time::Duration::from_secs(3));
    if !window_fallback.is_visible().unwrap_or(true) {
        log::warn!("[TIMING] Fallback: Showing window after 3s timeout");
        let _ = window_fallback.show();
    }
});
```

**How it works:**
- Backend listens for a `"window-ready"` event from the frontend
- When received, it calls `window.show()` to make the window visible
- **Fallback safety**: If the event isn't received within 3 seconds, show anyway (prevents stuck hidden window)

### 3. Frontend Ready Signal (`App.svelte`)

```javascript
// Track first render completion
afterUpdate(() => {
  if (!firstRenderComplete) {
    firstRenderComplete = true;

    // Schedule next frame check to confirm visual rendering
    requestAnimationFrame(async () => {
      // Emit window-ready event to backend to show the window
      try {
        const { emit } = await import('@tauri-apps/api/event');
        await emit('window-ready', { timestamp: performance.now() });
        console.log('[TIMING] Emitted window-ready event');
        console.log('[TIMING] === UI IS NOW VISIBLE ===');
      } catch (error) {
        console.error('[TIMING] Failed to emit window-ready event:', error);
      }
    });
  }
});
```

**How it works:**
- Uses `afterUpdate()` to detect when Svelte has finished the first render
- Uses `requestAnimationFrame()` to ensure the frame has actually painted
- Emits `"window-ready"` event to tell the backend the UI is ready
- **Result**: Window only shows after content is fully rendered

## Timeline Comparison

### Before Fix:
```
0ms    - Tauri window created and shown (black/empty)
0-500ms - User sees black screen
500ms  - WebView2 initializes
800ms  - JavaScript loads
1200ms - Svelte components render
1500ms - First paint (UI finally appears)
```

### After Fix:
```
0ms    - Tauri window created (hidden)
0-500ms - WebView2 initializes (user sees nothing)
500ms  - JavaScript loads (user sees nothing)
1200ms - Svelte components render (user sees nothing)
1500ms - First paint complete
1500ms - Frontend emits "window-ready"
1500ms - Backend receives event and shows window
1500ms - User sees fully rendered UI immediately!
```

## Benefits

1. **Eliminates black screen** - User never sees an empty window
2. **Better perceived performance** - Instant fully-rendered UI instead of gradual loading
3. **Graceful fallback** - 3-second timeout ensures window always shows eventually
4. **Maintains responsiveness** - User doesn't wait longer, just sees the result when ready
5. **Professional appearance** - No loading artifacts or blank frames

## Known Alternatives (Not Used)

### Hardware Acceleration Disable
- **Why not**: No official API in Tauri 2.x for Windows WebView2
- **Impact**: Would hurt performance significantly
- **Use case**: Only for graphics corruption issues, not initialization delays

### Transparent Window
- **Why not**: Can cause performance issues and compatibility problems
- **When useful**: For frameless windows with custom shapes
- **Our case**: Not needed, solid background color works better

### Loading Spinner
- **Why not**: Would require showing the window before content is ready (defeats the purpose)
- **Alternative**: Our fix makes the spinner unnecessary

## Testing the Fix

To verify the fix is working:

1. **Check logs** - Backend log should show:
   ```
   [TIMING] Received window-ready event: <time>
   [TIMING] Showing window now...
   [TIMING] Window shown successfully: <time>
   ```

2. **Check browser console** - Should show:
   ```
   [TIMING] First frame painted: <time> ms
   [TIMING] Emitted window-ready event: <time> ms
   [TIMING] === UI IS NOW VISIBLE ===
   ```

3. **Visual test** - Window should appear with fully rendered content, no black screen

4. **Fallback test** - If you break the event emission, window should still show after 3 seconds

## Troubleshooting

### Window doesn't appear at all
- **Check**: Is the fallback timeout working? (3 seconds)
- **Check**: Are there errors in the backend log?
- **Fix**: Increase fallback timeout or check event listener setup

### Window still shows briefly empty
- **Check**: Is the `afterUpdate` hook firing too early?
- **Check**: Browser console timing logs
- **Fix**: Add a small delay (50-100ms) before emitting event

### Window shows but content loads slowly
- **This is different**: The fix solves black screen, not slow rendering
- **Solution**: Use the performance logging to identify slow components
- **Optimization**: Lazy load components, reduce initial bundle size

## Related Resources

- [Tauri Issue #5170](https://github.com/tauri-apps/tauri/issues/5170) - Blank white screen on load
- [Tauri Issue #14068](https://github.com/tauri-apps/tauri/issues/14068) - Splash screen flashing
- [WebView2 Documentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/)
- [PERFORMANCE_LOGGING.md](./PERFORMANCE_LOGGING.md) - Performance timing guide
