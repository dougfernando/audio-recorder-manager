# Tauri UI Setup Guide

This guide explains how to set up, develop, and build the Tauri UI for the Audio Recorder Manager.

## Project Structure

```
audio-recorder-manager/
├── src/                      # Core library (audio recording logic)
│   ├── lib.rs               # Library entry point
│   ├── bin/
│   │   └── cli.rs           # CLI binary
│   └── ...                  # Other modules
├── src-tauri/               # Tauri backend
│   ├── src/
│   │   └── main.rs          # Tauri app with command handlers
│   ├── Cargo.toml           # Tauri dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   └── icons/               # Application icons
├── ui/                      # Svelte frontend
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/  # Svelte components
│   │   │   └── stores.js    # State management
│   │   ├── App.svelte       # Main app component
│   │   ├── main.js          # Entry point
│   │   └── app.css          # Global styles
│   ├── package.json         # Frontend dependencies
│   └── vite.config.js       # Vite configuration
└── Cargo.toml               # Root package config
```

## Prerequisites

### Required Software

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18 or later) and npm
   - Download from: https://nodejs.org/

3. **FFmpeg** (required for audio processing)
   - Windows: Download from https://www.gyan.dev/ffmpeg/builds/
   - Add FFmpeg to your system PATH

4. **Windows-specific requirements:**
   - Visual Studio Build Tools with "Desktop development with C++" workload
   - Or: MinGW-w64 toolchain (already configured in `.cargo/config.toml`)

### Rust Target for Windows

If you're cross-compiling or building for Windows:

```bash
rustup target add x86_64-pc-windows-gnu
```

## Installation & Setup

### 1. Install Frontend Dependencies

```bash
cd ui
npm install
```

### 2. Install Tauri CLI (optional, for additional commands)

```bash
npm install -g @tauri-apps/cli
# or
cargo install tauri-cli
```

### 3. Generate Application Icons (optional)

If you want to customize the app icon:

```bash
# Install tauri-cli globally if not already installed
npm install -g @tauri-apps/cli

# Generate icons from a source image (512x512 recommended)
cd src-tauri
tauri icon path/to/your-icon.png
```

This will generate all required icon formats in `src-tauri/icons/`.

## Development

### Running in Development Mode

You have two options for running the app in development:

**Option 1: Using npm (recommended)**

```bash
# From the ui directory
cd ui
npm run dev
```

Then in another terminal:

```bash
# From the root directory
cargo run --manifest-path src-tauri/Cargo.toml
```

**Option 2: Using Tauri CLI (if installed)**

```bash
# From the root directory
cargo tauri dev
# or
npm run tauri dev  # if you've added it to package.json
```

This will:
- Start the Vite dev server (frontend) on port 5173
- Build and run the Tauri app (backend)
- Enable hot-reloading for frontend changes

### Development Features

- **Hot Reload**: Frontend changes reload automatically
- **DevTools**: Press `F12` to open Chrome DevTools in the app
- **Rust Logs**: Set `RUST_LOG=debug` for detailed logging

```bash
RUST_LOG=debug cargo run --manifest-path src-tauri/Cargo.toml
```

## Building for Production

### Build the Application

```bash
# From the root directory
cd ui
npm run build

cd ../src-tauri
cargo build --release
```

Or use the Tauri CLI:

```bash
cargo tauri build
# or
npm run tauri build  # if configured in package.json
```

### Build Output

The built application will be in:
- **Executable**: `src-tauri/target/release/audio-recorder-manager-tauri.exe`
- **Installer**: `src-tauri/target/release/bundle/`
  - `nsis/`: NSIS installer (.exe)
  - `msi/`: Windows Installer package (.msi)

### Build Configuration

The build is configured in `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "targets": ["msi", "nsis"],
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    }
  }
}
```

## CLI Binary

The CLI binary is still available and can be built separately:

```bash
cargo build --release --bin audio-recorder-manager
```

Output: `target/release/audio-recorder-manager.exe`

This allows users to choose between:
- **GUI version**: Full Tauri application with UI
- **CLI version**: Command-line tool for scripting and automation

## Troubleshooting

### Build Errors

**Error: "can't find crate for `core`"**
- Solution: Install the target: `rustup target add x86_64-pc-windows-gnu`

**Error: "WebView2 not found"**
- Windows requires WebView2 runtime
- Download: https://developer.microsoft.com/microsoft-edge/webview2/
- Or set installer to bundle it (already configured)

**Error: "FFmpeg not found"**
- Ensure FFmpeg is in your system PATH
- Test: `ffmpeg -version`

### Frontend Issues

**Port 5173 already in use**
- Change port in `ui/vite.config.js` and `src-tauri/tauri.conf.json`

**Module not found errors**
- Delete `ui/node_modules` and run `npm install` again

### Development Mode Issues

**Backend not connecting to frontend**
- Ensure Vite dev server is running on port 5173
- Check `src-tauri/tauri.conf.json` → `build.devPath`

## Project Components

### Tauri Commands (Backend)

Located in `src-tauri/src/main.rs`:

- `start_recording()` - Start a new recording
- `stop_recording()` - Stop active recording
- `get_status()` - Get audio device status
- `recover_recordings()` - Recover interrupted recordings
- `get_recording_status()` - Get real-time recording status
- `list_recordings()` - List all recordings
- `get_active_sessions()` - Get active recording sessions

### Svelte Components (Frontend)

Located in `ui/src/lib/components/`:

- **RecordingPanel.svelte** - Recording controls and settings
- **ActiveRecording.svelte** - Real-time recording status display
- **DeviceStatus.svelte** - Audio device information
- **RecordingsList.svelte** - List of completed recordings
- **Recovery.svelte** - Recovery interface for interrupted recordings

### State Management

State is managed using Svelte stores in `ui/src/lib/stores.js`:

- Recording state (isRecording, currentSession, recordingStatus)
- Recordings list
- Device information
- UI preferences

### Real-time Updates

The application uses Tauri events for real-time status updates:

```javascript
// Frontend listens to backend events
listen('recording-status-update', (event) => {
  recordingStatus.set(event.payload);
});
```

The backend watches the status directory and emits events when status files change.

## Distribution

### Creating Installers

The build process creates two installer types:

1. **NSIS Installer** (.exe)
   - Simple installer wizard
   - Customizable install location
   - Start menu shortcuts

2. **MSI Installer** (.msi)
   - Windows Installer format
   - Enterprise-friendly
   - Silent install support

Both installers include:
- WebView2 bootstrapper (downloads if needed)
- Application files
- Start menu shortcut
- Uninstaller

### Signing (Optional)

For production distribution, sign your installers:

```json
// In src-tauri/tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

## Next Steps

1. **Customize Icons**: Replace placeholder icons with branded ones
2. **Test on Windows**: Build and test the complete application
3. **Add Features**: Extend UI with additional features as needed
4. **Create Releases**: Set up GitHub Actions for automated builds
5. **Documentation**: Add user documentation and help system

## Additional Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Svelte Documentation](https://svelte.dev/docs)
- [Vite Documentation](https://vitejs.dev/guide/)
- [Audio Recorder Manager CLI Docs](./README.md)

## Support

For issues or questions:
- Check existing GitHub issues
- Create a new issue with detailed information
- Include logs from both frontend (DevTools console) and backend (terminal)
