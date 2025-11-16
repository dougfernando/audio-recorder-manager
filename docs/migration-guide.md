# Migration Guide: Reorganization to Cargo Workspace

## Overview

The project has been reorganized from a flat structure to a Cargo workspace monorepo. This improves code organization, enables better code sharing, and consolidates storage between CLI and UI.

## What Changed

### Directory Structure

**Before:**
```
audio-recorder-manager/
├── src/              # CLI library + binary
├── src-tauri/        # Tauri app
├── ui/               # Svelte frontend
├── storage/          # CLI storage
└── Cargo.toml        # Single package
```

**After:**
```
audio-recorder-manager/
├── crates/
│   ├── core/         # Shared library
│   ├── cli/          # CLI binary
│   └── tauri-app/    # Tauri app (includes ui/)
├── storage/          # Shared storage for both
├── docs/             # Documentation
├── examples/         # Example scripts
└── Cargo.toml        # Workspace root
```

### Package Names

| Component | Old Name | New Name |
|-----------|----------|----------|
| Library | `audio_recorder_manager` | `audio_recorder_manager_core` |
| CLI Binary | `audio-recorder-manager` | `audio-recorder-manager` (unchanged) |
| Tauri App | `audio-recorder-manager-tauri` | `audio-recorder-manager-tauri` (unchanged) |

### Storage Location

- **Before**: CLI used `./storage/`, Tauri used `./src-tauri/storage/`
- **After**: Both use `./storage/` at workspace root
- **Benefit**: No data duplication, CLI and UI share recordings

### Import Paths

If you have custom code importing the library:

**Before:**
```rust
use audio_recorder_manager::config::RecorderConfig;
use audio_recorder_manager::commands::record;
```

**After:**
```rust
use audio_recorder_manager_core::config::RecorderConfig;
use audio_recorder_manager_core::commands::record;
```

## For Users

### CLI Usage (Unchanged)

The CLI binary name and usage remain identical:

```bash
# Still works exactly the same
audio-recorder-manager record 30 wav
audio-recorder-manager status
audio-recorder-manager stop
```

### Storage Migration

Your existing recordings are safe. To consolidate:

1. **Old CLI recordings**: Already in `./storage/` ✅
2. **Old Tauri recordings**: Located in `./src-tauri/storage/`

To merge Tauri recordings into the new shared storage:

```bash
# Windows
xcopy /s /i src-tauri\storage\recordings storage\recordings
xcopy /s /i src-tauri\storage\transcriptions storage\transcriptions

# Linux/macOS
cp -r src-tauri/storage/recordings/* storage/recordings/
cp -r src-tauri/storage/transcriptions/* storage/transcriptions/
```

After migration, old directories can be deleted (they're now gitignored).

## For Developers

### Building

**Before:**
```bash
# CLI
cargo build --release

# Tauri
cd src-tauri && cargo build --release
```

**After:**
```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p audio-recorder-manager-cli --release
cargo build -p audio-recorder-manager-tauri --release

# Or from subdirectory
cd crates/cli && cargo build --release
```

### Development Workflow

#### CLI Development

```bash
# Run CLI in debug mode
cargo run -p audio-recorder-manager-cli -- record 5 wav

# With logging
RUST_LOG=debug cargo run -p audio-recorder-manager-cli -- status

# Tests
cargo test -p audio-recorder-manager-core
```

#### Tauri Development

```bash
# Frontend
cd crates/tauri-app/ui
npm install
npm run dev

# Backend (in another terminal)
cd crates/tauri-app
cargo tauri dev

# Build for release
cd crates/tauri-app
npm run tauri build
```

#### Core Library Development

Changes to `crates/core/` automatically benefit both CLI and Tauri:

```bash
# Make changes to core
vim crates/core/src/recorder.rs

# Test with CLI
cargo run -p audio-recorder-manager-cli -- status

# Test with Tauri
cd crates/tauri-app && cargo tauri dev
```

### Configuration System

The new `RecorderConfig::get_workspace_storage_dir()` automatically locates the workspace root:

```rust
// In crates/core/src/config.rs
impl RecorderConfig {
    fn get_workspace_storage_dir() -> PathBuf {
        // Searches for Cargo.toml with [workspace]
        // Returns: /path/to/audio-recorder-manager/storage
    }
}
```

**No environment variables needed** - it just works!

### Adding Dependencies

Add to workspace in root `Cargo.toml`:

```toml
[workspace.dependencies]
new-crate = "1.0"
```

Then use in any crate:

```toml
# crates/core/Cargo.toml
[dependencies]
new-crate = { workspace = true }
```

### Git Workflow

The old `src/`, `src-tauri/`, and `ui/` directories are now gitignored. Ensure you:

1. Commit the new structure
2. The old directories won't be tracked
3. Keep `storage/` gitignored (recordings shouldn't be in git)

```bash
git status  # Should show clean after migration
git add .
git commit -m "Reorganize into Cargo workspace monorepo"
```

## Troubleshooting

### "Cannot find crate `audio_recorder_manager`"

Update import to `audio_recorder_manager_core`:

```diff
- use audio_recorder_manager::config::RecorderConfig;
+ use audio_recorder_manager_core::config::RecorderConfig;
```

### "Storage directory not found"

Ensure you're running from workspace root, or the auto-detection will find it:

```bash
# From workspace root
cargo run -p audio-recorder-manager-cli -- status  ✅

# From crates/cli (also works)
cd crates/cli
cargo run -- status  ✅
```

### Tauri build fails with "ui not found"

The UI is now in `crates/tauri-app/ui`:

```bash
cd crates/tauri-app/ui
npm install
cd ..
npm run tauri build
```

### Different recordings in CLI vs UI

If you see different files, you likely have old data in `src-tauri/storage/`. Follow the storage migration steps above.

## Benefits of New Structure

✅ **Single source of truth** - Core library used by both frontends
✅ **Unified storage** - No data duplication
✅ **Faster builds** - Workspace caches shared dependencies
✅ **Better organization** - Clear separation of concerns
✅ **Professional layout** - Follows Rust best practices
✅ **Easier testing** - Shared test infrastructure

## Rollback (If Needed)

Old files are preserved as `*.old`:

```bash
# Restore old structure (destructive!)
mv Cargo.toml.old Cargo.toml
mv Cargo.lock.old Cargo.lock

# Remove new structure
rm -rf crates/
```

**Note**: Not recommended - the new structure is better!

## Questions?

See [architecture.md](./architecture.md) for detailed design documentation.
