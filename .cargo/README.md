# Cargo Configuration

This directory contains cargo configuration files for different toolchains.

## Current: MSVC Toolchain (Default)

With no `config.toml` active, Cargo uses the default MSVC toolchain on Windows.
This is the recommended configuration for Tauri on Windows.

**Requirements:**
- Visual Studio Build Tools 2022 with "Desktop development with C++" ✅ (You have this installed!)
- Or Visual Studio with C++ workload

**No additional configuration needed** - MSVC works out of the box!

## Alternative: GNU Toolchain

If you need to use the GNU toolchain (MinGW-w64):

**To switch to GNU:**
```bash
# Activate GNU config
mv .cargo/config.toml.gnu .cargo/config.toml

# Clean build artifacts
cargo clean
cd src-tauri && cargo clean
```

**Requirements for GNU:**
- MinGW-w64 (latest version recommended)
- GCC in your PATH

## Available Config Files

- `config.toml.gnu` - GNU toolchain with linker flags to avoid .drectve errors
- `config.toml.backup` - Original simple GNU config (had linker issues)

## Switching Between Toolchains

**To use MSVC (current):**
```bash
# Ensure no config.toml exists (or rename it)
mv .cargo/config.toml .cargo/config.toml.gnu

# Clean build artifacts
cargo clean
cd src-tauri && cargo clean
```

**To use GNU:**
```bash
# Activate GNU config
mv .cargo/config.toml.gnu .cargo/config.toml

# Clean build artifacts
cargo clean
cd src-tauri && cargo clean
```

## Troubleshooting

**Always clean when switching toolchains:**
```bash
cargo clean
cd src-tauri && cargo clean
cd ..
```

This prevents mixing object files from different compilers.
