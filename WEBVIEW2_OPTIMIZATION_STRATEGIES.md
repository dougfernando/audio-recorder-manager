# WebView2 Optimization Strategies & Alternatives

## Problem Summary

WebView2 initialization is taking ~15 seconds, causing a poor user experience. This document explores optimization strategies and alternative approaches.

## Optimization Strategies

### 1. Pre-warming WebView2 (Most Effective)

**Concept**: Create a hidden WebView2 instance early to warm up the runtime, then reuse it.

**Implementation in Tauri**:

Unfortunately, Tauri doesn't expose a direct pre-warming API, but you can:

**Option A: Splash Screen Window**
```json
// tauri.conf.json
{
  "windows": [
    {
      "label": "splash",
      "title": "Loading...",
      "width": 400,
      "height": 300,
      "center": true,
      "resizable": false,
      "decorations": false,
      "url": "splash.html",  // Simple, fast-loading page
      "visible": true
    },
    {
      "label": "main",
      "title": "Audio Recorder Manager",
      "visible": false,
      // ... other config
    }
  ]
}
```

```rust
// main.rs setup handler
.setup(|app| {
    // Close splash after main window loads
    let splash = app.get_webview_window("splash").unwrap();
    let main = app.get_webview_window("main").unwrap();

    // Listen for main window ready
    main.once("tauri://created", move |_| {
        splash.close().unwrap();
        main.show().unwrap();
    });

    Ok(())
})
```

**Benefits**:
- First window initializes WebView2 runtime
- Main window loads much faster (runtime already initialized)
- Professional loading experience

---

### 2. Use Fixed WebView2 Runtime

**Problem**: Evergreen WebView2 checks for updates on initialization.

**Solution**: Bundle a specific WebView2 version.

```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "fixedRuntime",
        "path": "Microsoft.WebView2.FixedVersionRuntime.98.0.1108.50.x64"
      }
    }
  }
}
```

**Pros**:
- No update checks = faster initialization
- Consistent behavior across machines
- Offline installation works

**Cons**:
- Larger installer size (~150MB)
- Manual updates required
- Security patches need app updates

---

### 3. Optimize Browser Arguments

**Current setup**:
```json
"additionalBrowserArgs": "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"
```

**Additional aggressive optimizations**:
```json
"additionalBrowserArgs": "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-gpu-compositing --disable-smooth-scrolling --disable-backgrounding-occluded-windows --disable-renderer-backgrounding --disable-background-timer-throttling --disable-ipc-flooding-protection"
```

**What each does**:
- `--disable-gpu-compositing`: Skip GPU layer composition (faster start, slower rendering)
- `--disable-smooth-scrolling`: Disable smooth scrolling animation
- `--disable-backgrounding-occluded-windows`: Keep windows active when hidden
- `--disable-renderer-backgrounding`: Don't throttle background renderers
- `--disable-background-timer-throttling`: Don't throttle timers
- `--disable-ipc-flooding-protection`: Skip IPC flood checks

⚠️ **Warning**: These may impact runtime performance and security. Test thoroughly.

---

### 4. Profile Data Directory Optimization

**Problem**: WebView2 creates profile data, which can be slow on first run.

**Solution**: Pre-create and cache the profile directory.

```rust
// main.rs
use tauri::Manager;

.setup(|app| {
    // Set custom data directory on fast disk (SSD)
    let data_dir = app.path().app_data_dir()
        .expect("Failed to get app data dir")
        .join("webview2_profile");

    std::fs::create_dir_all(&data_dir).ok();

    // Tauri 2.x: This is handled automatically, but you can verify:
    log::info!("WebView2 profile directory: {:?}", data_dir);

    Ok(())
})
```

**Tips**:
- Ensure app data dir is on SSD, not HDD
- Pre-create directory structure in installer
- Check antivirus isn't scanning WebView2 profile

---

### 5. System-Level Optimizations

#### A. Update WebView2 Runtime

**Check version**:
```powershell
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv
```

**Update**:
- Download latest: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
- Install Evergreen Bootstrapper or Standalone Installer

#### B. Antivirus Exclusions

**Add to exclusions**:
1. Your app executable
2. WebView2 runtime folder: `C:\Program Files (x86)\Microsoft\EdgeWebView\Application`
3. App data directory

**Common culprits**:
- Windows Defender Real-time Protection
- Corporate antivirus (McAfee, Symantec, etc.)

#### C. Disable Windows Features

```powershell
# Disable SmartScreen (if acceptable for your use case)
Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer" -Name "SmartScreenEnabled" -Value "Off"

# Disable Windows Defender scanning for dev (NOT recommended for production)
Add-MpPreference -ExclusionPath "C:\Path\To\Your\App"
```

---

### 6. Code-Level Optimizations

#### A. Lazy Load Heavy Dependencies

```javascript
// Don't import everything at once
// BAD:
import everything from 'heavy-library';

// GOOD:
async function whenNeeded() {
  const { specific } = await import('heavy-library');
}
```

#### B. Minimize Initial Bundle

```javascript
// vite.config.js
export default {
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor': ['svelte'],
          'ui': ['./src/lib/components/*']
        }
      }
    }
  }
}
```

#### C. Preload Critical Assets

```html
<!-- index.html -->
<link rel="preload" href="/src/main.js" as="script">
<link rel="preload" href="/src/app.css" as="style">
<link rel="preconnect" href="http://localhost:5173">
```

---

## Alternative Technologies

### Option 1: Electron

**What it is**: Chromium + Node.js bundled

**Pros**:
- ✅ Full control over Chromium version
- ✅ Extensive ecosystem and tooling
- ✅ Consistent across Windows, Mac, Linux
- ✅ Mature, widely used

**Cons**:
- ❌ Large bundle size (~150MB+)
- ❌ Higher memory usage
- ❌ Slower startup than native
- ❌ Security concerns (full Node.js access)

**When to use**: Need maximum compatibility and ecosystem support.

---

### Option 2: Native Windows UI (Tauri with Native Rendering)

**Tauri is exploring native rendering**: https://github.com/tauri-apps/tauri/discussions/6926

**Current options**:

#### A. WinUI 3 / Windows App SDK
- Modern Windows native UI
- Fluent Design System
- Fast, native performance

#### B. Slint (with Tauri backend)
- Native Rust UI framework
- GPU-accelerated rendering
- Cross-platform

```toml
# Cargo.toml
[dependencies]
slint = "1.0"
```

**Pros**:
- ✅ Instant startup (no browser engine)
- ✅ Native look and feel
- ✅ Lower memory usage
- ✅ Better performance

**Cons**:
- ❌ Lose web technologies (HTML/CSS/JS)
- ❌ Different UI for each platform
- ❌ Smaller ecosystem
- ❌ More platform-specific code

---

### Option 3: Tauri with Alternative WebView

**Platform-specific webviews**:

#### Windows: MSHTML (Internet Explorer Engine)
- **Deprecated**, don't use

#### Windows: WebView2 is the only modern option
- Edge Chromium-based
- This is what we're already using

#### Alternative: Run in Browser
- Launch system default browser
- Connect via WebSocket/HTTP
- No embedded webview needed

```rust
// Example: Browser-based approach
tauri::Builder::default()
    .setup(|app| {
        // Launch system browser
        let url = "http://localhost:3000";
        opener::open(url)?;
        Ok(())
    })
```

**Pros**:
- ✅ Zero WebView2 initialization time
- ✅ User's preferred browser
- ✅ Always up-to-date browser

**Cons**:
- ❌ Not a native app experience
- ❌ Security: need to secure local server
- ❌ Can't control browser environment

---

### Option 4: Progressive Web App (PWA)

**Concept**: Web app that can be "installed" to desktop.

**Implementation**:
1. Build as regular web app
2. Add service worker
3. Add manifest.json
4. Users install via browser

**Pros**:
- ✅ No installation needed
- ✅ Automatic updates
- ✅ Works everywhere
- ✅ Zero startup time (after first load)

**Cons**:
- ❌ Limited system access
- ❌ Requires internet (unless cached)
- ❌ Browser-dependent experience
- ❌ No native integration

---

## Recommendations

### For Your Use Case (Audio Recorder)

Given you need system audio access and file I/O:

**Best Option: Stick with Tauri + WebView2, but optimize**

**Implementation Priority**:

1. **🔥 High Priority - Implement Splash Screen**
   - Pre-warms WebView2
   - Professional loading experience
   - Easiest to implement

2. **🔥 High Priority - System Optimizations**
   - Update WebView2 runtime
   - Add antivirus exclusions
   - Check for slow disk I/O

3. **⚡ Medium Priority - Fixed Runtime**
   - Bundle specific WebView2 version
   - Eliminates update checks
   - More control

4. **⚡ Medium Priority - Aggressive Browser Args**
   - Test additional flags
   - Measure impact
   - May help 1-2 seconds

5. **💡 Low Priority - Alternative Technologies**
   - Only if WebView2 absolutely won't work
   - Requires significant refactoring
   - Loss of web development benefits

---

## Expected Results

**Current**:
- 15 seconds to setup handler
- Poor user experience

**With Splash Screen + Optimizations**:
- Splash appears instantly
- Main window ready in 2-5 seconds
- Professional experience

**With Fixed Runtime**:
- Additional 1-2 second improvement
- More consistent performance

**With All Optimizations**:
- Target: Main UI visible in <3 seconds
- Realistic: 5-7 seconds (still much better)

---

## Testing Checklist

After implementing optimizations:

- [ ] Time from app launch to splash visible
- [ ] Time from splash to main window visible
- [ ] Browser console logs show no errors
- [ ] Backend logs show improved timing
- [ ] Antivirus is not interfering
- [ ] Test on clean Windows install
- [ ] Test on slow hardware
- [ ] Measure memory usage
- [ ] Check app size

---

## Additional Resources

- [WebView2 Performance Guide](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance)
- [Tauri v2 Performance](https://v2.tauri.app/concept/performance/)
- [Electron vs Tauri Comparison](https://blog.logrocket.com/tauri-vs-electron-comparison-migration-guide/)
- [Chromium Command Line Switches](https://peter.sh/experiments/chromium-command-line-switches/)
- [WebView2 Runtime Versions](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
