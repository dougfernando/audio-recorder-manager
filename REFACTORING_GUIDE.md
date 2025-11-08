# Rust Best Practices Refactoring Guide

This document outlines the architectural improvements implemented and planned for the audio-recorder-manager project to follow Rust best practices.

## Completed Foundational Modules

### 1. Error Handling (`src/error.rs`)
**Status**: ✅ Implemented

Custom error types using `thiserror` crate provide:
- Type-safe error handling with `RecorderError` enum
- Automatic error conversions from `io::Error` and `serde_json::Error`
- Clear error categories (DeviceError, RecordingError, ConversionError, InvalidParameter)
- Better debugging and error propagation

**Usage**:
```rust
use crate::error::{RecorderError, Result};

fn my_function() -> Result<()> {
    // Returns Result<(), RecorderError>
}
```

### 2. Configuration Management (`src/config.rs`)
**Status**: ✅ Implemented

Centralized configuration with `RecorderConfig`:
- Single source of truth for all configuration values
- No more hardcoded paths or magic numbers
- Easy to extend with environment variables or config files
- Helper method `ensure_directories()` for setup

**Usage**:
```rust
let config = RecorderConfig::new();
config.ensure_directories()?;
```

### 3. Domain Models (`src/domain.rs`)
**Status**: ✅ Implemented

Type-safe domain models:
- `AudioFormat` enum with `FromStr` implementation for parsing
- `RecordingDuration` enum to handle both fixed and manual modes
- `SessionId` newtype for type safety
- `RecordingSession` to encapsulate session data

**Benefits**:
- Can't accidentally mix up session IDs and filenames
- Type-driven development
- Self-documenting code
- Compile-time guarantees

## Next Steps for Full Implementation

### Phase 1: Update Existing Code (Priority: HIGH)

1. **Update `main.rs`** to use new types:
   ```rust
   // OLD
   let audio_format = if args.len() > 3 {
       args[3].to_lowercase()
   } else {
       "wav".to_string()
   };

   // NEW
   use crate::domain::AudioFormat;
   let audio_format = if args.len() > 3 {
       AudioFormat::from_str(&args[3])?
   } else {
       AudioFormat::Wav
   };
   ```

2. **Update `recorder.rs`** to use `RecorderConfig`:
   ```rust
   // Pass config instead of individual paths
   pub async fn record(
       session: RecordingSession,
       config: &RecorderConfig,
   ) -> Result<PathBuf> {
       // ...
   }
   ```

3. **Update `wasapi_loopback.rs`** to use custom errors:
   ```rust
   // OLD
   use anyhow::{Context, Result};

   // NEW
   use crate::error::{RecorderError, Result};

   // Convert Windows API errors
   .map_err(|e| RecorderError::DeviceError(format!("Failed to initialize: {:?}", e)))?
   ```

### Phase 2: Status Observer Pattern (Priority: MEDIUM)

Create `src/status/` module:

```rust
// src/status/observer.rs
pub trait StatusObserver: Send + Sync {
    fn on_progress(&self, status: RecordingStatus);
    fn on_complete(&self, result: RecordingResult);
}

// src/status/json.rs
pub struct JsonFileObserver {
    status_dir: PathBuf,
}

// src/status/terminal.rs
pub struct TerminalObserver;
```

**Benefits**:
- Separates status reporting from recording logic
- Easy to add new observers (websockets, databases, etc.)
- Each observer has single responsibility

### Phase 3: Trait-Based Recorder (Priority: MEDIUM)

Create `src/recorder/traits.rs`:

```rust
#[async_trait]
pub trait AudioRecorder: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<RecordingMetrics>;
    fn get_status(&self) -> RecorderStatus;
}
```

**Benefits**:
- Platform-agnostic interface
- Easy to test with mock implementations
- Clear abstraction boundaries

### Phase 4: File Reorganization (Priority: LOW)

```
src/
├── main.rs              # Entry point only
├── error.rs            # ✅ Done
├── config.rs           # ✅ Done
├── domain.rs           # ✅ Done
├── cli/
│   ├── mod.rs          # CLI coordinator
│   ├── commands.rs     # Command definitions
│   └── parser.rs       # Argument parsing
├── recorder/
│   ├── mod.rs          # Public interface
│   ├── traits.rs       # AudioRecorder trait
│   ├── wasapi.rs       # Windows implementation
│   ├── cpal.rs         # Cross-platform implementation
│   └── state.rs        # RecorderState
├── conversion/
│   ├── mod.rs
│   └── m4a.rs          # WAV to M4A conversion
├── status/
│   ├── mod.rs
│   ├── observer.rs     # StatusObserver trait
│   ├── json.rs         # JsonFileObserver
│   └── terminal.rs     # TerminalObserver
└── devices/
    ├── mod.rs
    └── manager.rs      # DeviceManager
```

## Performance Optimizations

### String Allocation in Hot Paths

```rust
// Instead of creating new strings every second
use std::fmt::Write;

let mut buffer = String::with_capacity(256);
write!(&mut buffer, "[Recording] Progress: {}%...", progress)?;
eprintln!("{}", buffer);
buffer.clear();  // Reuse for next iteration
```

### Cross-Thread Communication

```rust
// Use channels instead of Arc<AtomicU64>
use tokio::sync::watch;

let (tx, rx) = watch::channel(RecordingMetrics::default());
// Recording thread sends updates
tx.send(new_metrics)?;
// Main thread receives
let metrics = rx.borrow().clone();
```

## Migration Strategy

### Incremental Approach (Recommended)

1. ✅ Add new modules without breaking existing code
2. Update one module at a time (start with `main.rs`)
3. Run tests after each change
4. Keep old code working until new code is verified
5. Remove old code once migration is complete

### Testing Strategy

1. Create integration tests for current behavior
2. Refactor while keeping tests passing
3. Add unit tests for new modules
4. Use mock implementations for testing

## Benefits Summary

### Maintenance
- **Clear module boundaries**: Each module has single responsibility
- **Type safety**: Compiler catches errors early
- **Self-documenting**: Types explain intent
- **Easier debugging**: Custom errors with context

### Performance
- **Zero-cost abstractions**: Rust traits compile to efficient code
- **Reduced allocations**: Reusable buffers
- **Better concurrency**: Proper channel usage

### Scalability
- **Easy to extend**: Add new observers, formats, recorders
- **Testable**: Mock implementations for testing
- **Configurable**: Central configuration management

## Current Status

### Completed
- ✅ **Phase 0**: Error handling foundation, Configuration management, Domain models
- ✅ **Phase 1**: Integration with existing code - main.rs now uses all new types
- ✅ **Phase 2**: Status observer pattern - Flexible, extensible status reporting
- ✅ **Phase 3**: Evaluated and determined unnecessary - recorders already have consistent interfaces
- ✅ **Phase 4**: Evaluated and determined unnecessary - current file organization is clean and logical

### Implementation Summary
The core refactoring is complete! The codebase now follows Rust best practices with:
- Type-safe domain models (AudioFormat, RecordingDuration, RecordingSession)
- Centralized configuration (RecorderConfig)
- Custom error types (RecorderError)
- Observer pattern for status reporting (StatusObserver, JsonFileObserver)
- Clear separation of concerns
- Extensible architecture
- Consistent recorder interfaces (both WASAPI and CPAL have matching methods)
- Logical file organization with clear module boundaries

### Phases 3 & 4 Evaluation

**Phase 3 (Trait-Based Recorder)**: Upon analysis, both the WASAPI and CPAL recorders already provide consistent interfaces with methods like:
- `get_sample_rate()` - Returns audio sample rate
- `get_channels()` - Returns number of audio channels
- `get_frames_captured()` - Returns frames captured count
- `has_audio_detected()` - Returns whether audio was detected
- `stop()` - Stops the recording

The recorders are platform-specific and use `#[cfg(windows)]` attributes, making trait abstraction unnecessary. The current approach provides clean separation without adding complexity.

**Phase 4 (File Reorganization)**: The current file structure is already clean and follows Rust conventions:
- Root level modules for major features (error, config, domain)
- Nested modules where appropriate (status/)
- Clear naming and logical grouping
- Easy to navigate and understand

Further reorganization would add complexity without meaningful benefits. The current structure already provides good separation of concerns and is easy to maintain.

All features are fully functional and tested. The refactoring goals have been achieved.

## Notes

- All new modules are backward compatible
- Existing functionality continues to work
- Can be adopted incrementally
- No breaking changes until explicitly migrated
