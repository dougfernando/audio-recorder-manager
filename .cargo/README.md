# Cargo Configuration

This directory contains cargo configuration files for different toolchains.

## Current: GNU Toolchain (config.toml)

The project is now configured to use `x86_64-pc-windows-gnu` with special linker flags
to avoid `.drectve` symbol errors that can occur with MinGW.

**Requirements:**
- MinGW-w64 (latest version recommended)
- GCC in your PATH

**If you still get linker errors**, try:
1. Update MinGW-w64 to the latest version
2. Clean build: `cargo clean && cd src-tauri && cargo clean`
3. Or switch to MSVC (see below)

## Alternative: MSVC Toolchain (Recommended if GNU causes issues)

The MSVC toolchain is officially recommended by Tauri and generally more stable on Windows.

**To switch to MSVC:**
```bash
# Rename or delete config.toml
mv .cargo/config.toml .cargo/config.toml.disabled

# Clean build artifacts
cargo clean
cd src-tauri && cargo clean
```

**Requirements:**
- Visual Studio Build Tools with "Desktop development with C++"
- Or Visual Studio with C++ workload
- Download: https://visualstudio.microsoft.com/downloads/

## Backup Files

- `config.toml.backup` - Original simple GNU config (had linker issues)
- `config.toml.disabled` - For temporarily disabling config

## Troubleshooting

**If switching between toolchains:**
Always clean build artifacts to avoid mixing object files:
```bash
cargo clean
cd src-tauri && cargo clean
cd ..
```

**If GNU toolchain has linker errors:**
The `.drectve` warnings are usually harmless, but if you get `collect2.exe: error`,
either update MinGW or switch to MSVC.
