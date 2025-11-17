# WebView2 Initialization Delay - Analysis & Fix

## Problem Identified

Based on the timing logs, the real bottleneck is **NOT** during UI rendering, but during **WebView2/Window creation itself**.

### Timing Analysis

```
[TIMING] Starting Tauri application run loop: 2.8501ms
[TIMING] Setup handler executing: 15.2183348s  ← 15 SECONDS LATER!
```

**Key Finding**: There's a **15-second delay** between Tauri starting the run loop and the setup handler executing. This delay occurs during:
1. Window creation
2. WebView2 runtime initialization
3. Initial WebView loading

### Why the Previous Fix Didn't Work

The deferred window visibility approach (`visible: false`) didn't help because:
- The delay happens **before** the setup handler even runs
- The window is being created during this 15-second period (even when hidden)
- WebView2 initialization is the bottleneck, not the UI rendering

### Secondary Issue

The frontend never emitted the `window-ready` event, causing the fallback timeout to trigger:
```
[TIMING] Setup handler complete: 15.2877325s
[TIMING] Fallback: Showing window after 3s timeout: 18.288644s
```

This suggests the Svelte app loaded but had issues with the event emission.

## Root Cause: WebView2 Initialization

WebView2 (Microsoft Edge WebView) can be slow to initialize on Windows due to:

1. **Runtime Loading**: Loading the Edge WebView2 runtime DLLs
2. **Process Creation**: Creating the WebView2 browser process
3. **Security Checks**: Windows SmartScreen and security features
4. **Network Checks**: Some WebView2 features may check for updates
5. **GPU Initialization**: Hardware acceleration setup

## The Fix

### 1. Disable Unnecessary WebView2 Features

Added browser arguments to disable features that cause initialization delays:

**`tauri.conf.json`**:
```json
{
  "additionalBrowserArgs": "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"
}
```

**What these disable**:
- `msWebOOUI`: Microsoft Web Out-of-Browser UI features
- `msPdfOOUI`: PDF rendering features (we don't need this)
- `msSmartScreenProtection`: SmartScreen checking (can cause delays)

### 2. Show Window Immediately

Since hiding the window doesn't prevent the delay, we set it to visible:

**`tauri.conf.json`**:
```json
{
  "visible": true,
  "backgroundColor": "#F5F5F7"  // Match app background
}
```

**Benefits**:
- User sees the window with background color during initialization
- Better than nothing for 15 seconds
- `backgroundColor` provides a smooth transition when UI loads

### 3. Simplified Setup Handler

Removed the window-ready event listener complexity since:
- The delay happens before the setup handler runs
- The frontend wasn't reliably emitting the event
- Simpler code is more maintainable

## Expected Improvement

### Before:
```
0s     - App starts
15s    - Window finally created (user sees black/empty screen)
18s    - Fallback timeout shows window
18s+   - UI renders
```

### After:
```
0s     - App starts
0s     - Window shows with background color #F5F5F7
15s    - WebView2 initialized (hopefully faster with disabled features)
15s+   - UI renders and displays
```

## Additional Optimization Options

If the delay persists, consider these additional approaches:

### 1. **WebView2 Runtime Optimization**

Ensure you're using the latest WebView2 runtime:
```bash
# Check WebView2 version
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv
```

Update to the latest evergreen version if needed.

### 2. **Disable Hardware Acceleration** (Last Resort)

If GPU initialization is causing delays:
```json
{
  "additionalBrowserArgs": "--disable-gpu --disable-software-rasterizer"
}
```

⚠️ **Warning**: This will impact performance. Only use if absolutely necessary.

### 3. **Use Fixed WebView2 Runtime** (Advanced)

Instead of the evergreen runtime, bundle a fixed version:
```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "fixedRuntime",
        "path": "path/to/webview2/runtime"
      }
    }
  }
}
```

### 4. **Preload Critical Resources**

Add to `index.html`:
```html
<link rel="preload" href="/src/main.js" as="script">
<link rel="preload" href="/src/app.css" as="style">
```

### 5. **Check for Antivirus Interference**

Some antivirus software scans WebView2 processes, causing delays:
- Temporarily disable antivirus to test
- Add Tauri app to antivirus exclusions if needed

## Testing the Fix

### 1. Check Timing Logs

Backend log should show improvement:
```
[TIMING] Starting Tauri application run loop: ~3ms
[TIMING] Setup handler executing: <should be faster than 15s>
```

### 2. Browser Console

Frontend should load without errors:
```
[TIMING] App.svelte script starting to execute
[TIMING] All components imported: <time> ms
[TIMING] First frame painted: <time> ms
```

### 3. Visual Test

- Window should appear with light gray background (#F5F5F7)
- UI should load as soon as WebView2 is initialized
- No more 15-second blank waiting period

## Troubleshooting

### Issue: Still seeing 15-second delay

**Try**:
1. Update WebView2 runtime to latest version
2. Check Windows Event Viewer for WebView2 errors
3. Disable antivirus temporarily
4. Try `--disable-gpu` browser arg

### Issue: Window shows but stays blank

**Check**:
1. Browser console for JavaScript errors (F12)
2. Frontend timing logs for where it's stuck
3. Network tab for failed resource loads

### Issue: Faster but UI takes long to render

**Different problem**:
- This is frontend rendering delay, not WebView2
- Use the performance logging to identify slow components
- Consider lazy loading or code splitting

## Related Resources

- [WebView2 Performance Best Practices](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance)
- [Tauri Window Configuration](https://tauri.app/v1/api/config/#windowconfig)
- [Chromium Command Line Switches](https://peter.sh/experiments/chromium-command-line-switches/)
- [PERFORMANCE_LOGGING.md](./PERFORMANCE_LOGGING.md) - Performance timing guide

## Summary

The black screen delay was caused by **WebView2 initialization taking 15 seconds**, not by UI rendering. The fix involves:

1. ✅ Disabling unnecessary WebView2 features
2. ✅ Showing window immediately with matching background color
3. ✅ Simplified setup handler

This should significantly reduce the initialization time and provide a better user experience during the remaining delay.
