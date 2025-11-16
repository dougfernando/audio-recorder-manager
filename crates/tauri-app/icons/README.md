# Application Icons

This directory should contain the application icons for Tauri.

## Required Icon Files

For a proper Tauri build, you need the following icon files:

- `32x32.png` - 32x32 pixel PNG icon
- `128x128.png` - 128x128 pixel PNG icon
- `128x128@2x.png` - 256x256 pixel PNG icon (retina)
- `icon.icns` - macOS icon file
- `icon.ico` - Windows icon file

## Generating Icons

You can use the `@tauri-apps/cli` to generate all required icons from a single source image:

```bash
npm install -g @tauri-apps/cli
tauri icon path/to/your-icon.png
```

The source image should be:
- At least 512x512 pixels
- Square aspect ratio
- PNG format with transparency
- Simple, recognizable design

## Temporary Placeholder

For development purposes, you can use placeholder icons. For production, please replace these with proper branded icons for your application.

## Icon Design Guidelines

- Use a simple, recognizable symbol (e.g., microphone or waveform)
- Ensure good contrast for visibility
- Test at small sizes (16x16, 32x32) to ensure clarity
- Use transparency for better integration with OS themes
