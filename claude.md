# Claude Development Guidelines for Audio Recorder Manager

## Critical: Antivirus Considerations

**⚠️ NEVER propose or implement changes that may trigger antivirus false positives.**

This is a legitimate audio recording application that must maintain a clean security profile. Any code that resembles malware behavior will cause major user friction and damage trust.

### 🚫 Prohibited Patterns

#### 1. **Process Enumeration/Monitoring**
- ❌ Never use `tasklist`, `ps`, `Get-Process` or similar commands
- ❌ Never query running processes by PID
- ❌ Never check if specific processes are running (especially security software)
- ✅ Instead: Use natural error detection (pipe failures, exit codes)

#### 2. **Hidden Process Execution**
- ⚠️ Use `CREATE_NO_WINDOW` flag sparingly and only when absolutely necessary
- ✅ Document why it's needed (e.g., preventing FFmpeg console flashing)
- ❌ Never hide processes that don't have legitimate UI concerns

#### 3. **System Monitoring**
- ❌ No keyboard/mouse logging or monitoring
- ❌ No screenshot or screen recording (except explicit user-initiated audio)
- ❌ No system-wide hooks or injection

#### 4. **Network Activity**
- ❌ No undisclosed network connections
- ❌ No process injection or remote code execution
- ✅ Any network features must be explicitly user-requested and transparent

#### 5. **File System Operations**
- ❌ No modifications to system directories
- ❌ No executable dropping or self-modification
- ✅ Only write to user's designated storage directories

### ✅ Approved Patterns

#### Safe Process Interaction
```rust
// ✅ GOOD: Natural failure detection via I/O errors
match stdin.write_all(&bytes) {
    Ok(_) => { /* continue */ }
    Err(e) => { /* process died, handle gracefully */ }
}

// ❌ BAD: Process monitoring via tasklist
Command::new("tasklist").args(["/FI", "PID eq 1234"])
```

#### Safe Error Handling
```rust
// ✅ GOOD: Wait for process exit status
let status = process.wait()?;
if !status.success() { /* handle error */ }

// ❌ BAD: Active process health checking
loop {
    if is_process_alive(pid) { /* ... */ }
}
```

### 🔍 Review Checklist

Before proposing any code changes, verify:

- [ ] No external process querying or monitoring
- [ ] No hidden window creation (unless strictly necessary and documented)
- [ ] No system-wide API hooks or injection
- [ ] No memory scanning or process manipulation
- [ ] No obfuscation or anti-debugging techniques
- [ ] All Windows API calls are justified and documented
- [ ] All unsafe code blocks are minimal and necessary

### 📝 Windows API Usage Guidelines

When using Windows APIs (WASAPI, COM, etc.):

1. **Document the purpose** - Explain why the API call is necessary
2. **Minimize unsafe blocks** - Keep unsafe code as small as possible
3. **Use standard patterns** - Follow Microsoft's official examples
4. **No low-level hooks** - Avoid SetWindowsHookEx, kernel32 injection, etc.

### 🎯 This Project's Legitimate Requirements

The following are **approved and necessary** for audio recording:

✅ **WASAPI Audio Capture**
- `CoInitializeEx` for COM initialization
- `IMMDeviceEnumerator` for audio device access
- `IAudioClient` for audio streaming
- **Why needed**: Windows audio recording API

✅ **FFmpeg Process Spawning**
- `Command::new("ffmpeg")` with piped stdin/stdout
- `CREATE_NO_WINDOW` flag to prevent console flashing
- **Why needed**: Audio format conversion and merging

✅ **File I/O in User Storage**
- Writing to `recordings/` and `transcriptions/` directories
- **Why needed**: Saving user's recordings

### 🚨 When In Doubt

If you're unsure whether a pattern might trigger antivirus:

1. **Ask the user first** before implementing
2. **Explain the trade-offs** (functionality vs. security profile)
3. **Provide alternatives** with cleaner security profiles
4. **Prefer simplicity** - complex patterns are more suspicious

### 📚 Historical Issues

**Past problems we've fixed:**
- ❌ `tasklist` process monitoring
  - **Why flagged**: Process enumeration is classic malware behavior
  - **Solution**: Detect FFmpeg failures via pipe write errors instead

---

**Remember:** A clean security profile is just as important as functionality. False positives cause user abandonment and support burden.
