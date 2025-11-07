# Setup Guide for Windows

## Linker Issue on Windows

You're encountering a known issue where Git's `link.exe` (Unix symlink tool) conflicts with the MSVC linker that Rust needs.

### Solution 1: Use GNU Toolchain (Recommended for Quick Setup)

1. Remove the `.cargo/config.toml` file if it exists
2. Build with explicit target:
```bash
cargo build --target x86_64-pc-windows-gnu
```

### Solution 2: Fix PATH Order

Temporarily remove Git from PATH before building:
```bash
$env:PATH = ($env:PATH -split ';' | Where-Object { $_ -notlike '*Git*' }) -join ';'
cargo build
```

### Solution 3: Install Visual Studio Build Tools

1. Download Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/
2. Install "Desktop development with C++" workload
3. Restart terminal
4. Build normally: `cargo build`

### Solution 4: Use WSL2 (Recommended for Development)

If you have WSL2 installed:
```bash
# In WSL2
cd /mnt/c/Users/douglas.f.silva/Projects/audio-recorder-manager
cargo build
```

## Current Workaround

For now, use the GNU toolchain explicitly:
```bash
# Check
cargo check --target x86_64-pc-windows-gnu

# Build
cargo build --target x86_64-pc-windows-gnu --release

# Run
cargo run --target x86_64-pc-windows-gnu -- record 5 wav
```

The binary will be in: `target/x86_64-pc-windows-gnu/release/audio-recorder-manager.exe`
