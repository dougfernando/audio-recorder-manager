# Cargo Configuration

This directory contains cargo configuration files.

## config.toml.backup

The original `config.toml` was configured for GNU toolchain (`x86_64-pc-windows-gnu`).
This has been backed up because it caused linker issues with Tauri on Windows.

For Tauri on Windows, the MSVC toolchain is recommended and more reliable.

## Using MSVC Toolchain (Recommended for Tauri)

With no `config.toml`, Cargo will use the default MSVC toolchain on Windows.

Requirements:
- Visual Studio Build Tools with "Desktop development with C++"
- Or Visual Studio with C++ workload

## Reverting to GNU if Needed

If you need the GNU toolchain for CLI builds:
```bash
mv config.toml.backup config.toml
```

Note: You may need to clean build artifacts when switching:
```bash
cargo clean
cd src-tauri && cargo clean
```
