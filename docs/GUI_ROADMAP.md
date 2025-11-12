# GUI Implementation Roadmap

## Overview

This roadmap breaks down the GUI implementation from [GUI_PLAN.md](GUI_PLAN.md) into manageable, incremental phases. Each phase builds upon the previous one and can be completed, tested, and released independently.

## Current Status: Foundation Complete ✅

- [x] Dual binary architecture
- [x] Library pattern for code sharing
- [x] GUI module scaffolding
- [x] Basic state management structure
- [x] Minimal proof-of-concept GUI

## Phase 0: Proof of Concept (Current)

**Status**: ✅ Complete

**Goal**: Validate the dual-binary architecture with a minimal working GUI

**Deliverables**:
- [x] Simple window that opens
- [x] Basic GPUI app structure
- [x] Display static text
- [x] Verify CLI still works
- [x] Build process documented

**Duration**: 1 day

**Files**:
- `src/gui/main.rs` - Entry point
- `src/gui/app.rs` - Placeholder UI
- `src/gui/state/app_state.rs` - Basic state

---

## Phase 1: Navigation & Layout Foundation

**Status**: 📋 Planned

**Goal**: Implement the main window layout with sidebar navigation

**Deliverables**:
- [ ] Main window with sidebar + content area
- [ ] Sidebar with 5 navigation buttons (Record, Monitor, History, Recovery, Settings)
- [ ] Active panel highlighting
- [ ] Panel switching (empty panels for now)
- [ ] Basic theme colors
- [ ] Window size persistence

**Estimated Effort**: 3-5 days

**Key Files to Create**:
```
src/gui/
├── components/
│   ├── sidebar.rs          # Navigation sidebar
│   ├── panel_container.rs  # Content area wrapper
│   └── nav_button.rs       # Navigation button component
└── theme.rs                # Theme system
```

**Implementation Steps**:
1. Create `theme.rs` with color palette
2. Implement `Sidebar` component with navigation buttons
3. Add click handlers to switch panels
4. Create empty placeholder for each panel
5. Style active/inactive states
6. Test navigation flow

**Dependencies**: None (uses only GPUI)

**Success Criteria**:
- Can click between all 5 panels
- Active panel is visually highlighted
- Window opens at reasonable size
- Layout looks clean and organized

---

## Phase 2: Record Panel - Basic

**Status**: 📋 Planned

**Goal**: Implement recording configuration and start functionality

**Deliverables**:
- [ ] Duration input field (number input)
- [ ] Manual/Fixed mode radio buttons
- [ ] Format selection (WAV/M4A)
- [ ] Quality preset dropdown
- [ ] Start Recording button
- [ ] Preview of output filename
- [ ] Start recording on button click
- [ ] Transition to Monitor panel

**Estimated Effort**: 5-7 days

**Key Files to Create**:
```
src/gui/
├── components/
│   ├── record_panel.rs     # Main record configuration panel
│   ├── input_field.rs      # Number input component
│   ├── radio_group.rs      # Radio button group
│   ├── dropdown.rs         # Dropdown selector
│   └── button.rs           # Primary button component
└── services/
    └── recorder_service.rs # Wrap CLI record command
```

**Implementation Steps**:
1. Create form input components (text, radio, dropdown)
2. Layout record panel with all inputs
3. Add state binding for form values
4. Implement preview text generation
5. Create `RecorderService` to wrap `commands::record::execute()`
6. Wire up Start button to spawn recording
7. Navigate to Monitor panel on start
8. Handle errors with user feedback

**Dependencies**:
- Phase 1 (navigation)
- Core library (existing commands)

**Success Criteria**:
- Can configure all recording parameters
- Preview shows correct filename
- Clicking Start actually starts a recording
- Automatically switches to Monitor panel
- Errors display to user

---

## Phase 3: File Watcher Service

**Status**: 📋 Planned

**Goal**: Implement real-time file watching for status updates

**Deliverables**:
- [ ] File watcher service using `notify`
- [ ] Watch status directory for changes
- [ ] Parse status JSON files
- [ ] Emit events on status updates
- [ ] Debouncing (500ms)
- [ ] Error handling for missing files

**Estimated Effort**: 3-4 days

**Key Files to Create**:
```
src/gui/services/
├── mod.rs
├── file_watcher.rs         # Watch status/recordings dirs
└── events.rs               # Event types
```

**Implementation Steps**:
1. Create `FileWatcherService` struct
2. Set up `notify` file watcher
3. Watch `storage/status/` directory
4. On file change, read and parse JSON
5. Emit `RecordingStatusUpdated` event
6. Add debouncing to prevent spam
7. Handle file not found gracefully
8. Test with manual file changes

**Dependencies**:
- `notify` crate
- `notify-debouncer-full` crate

**Success Criteria**:
- Detects new status files immediately
- Parses JSON correctly
- Debounces rapid changes
- Handles errors gracefully
- No performance impact

---

## Phase 4: Monitor Panel - Real-time Updates

**Status**: 📋 Planned

**Goal**: Display live recording status with progress and audio levels

**Deliverables**:
- [ ] Session info display (ID, filename)
- [ ] Progress bar (visual + percentage)
- [ ] Time display (elapsed / total)
- [ ] System audio level indicator
- [ ] Microphone level indicator
- [ ] Status badges ("Audio Detected" / "Silent")
- [ ] Stop button
- [ ] Real-time updates from file watcher
- [ ] Completion handling

**Estimated Effort**: 5-7 days

**Key Files to Create**:
```
src/gui/components/
├── monitor_panel.rs        # Main monitor display
├── progress_bar.rs         # Progress bar component
├── level_meter.rs          # Audio level meter
└── status_badge.rs         # Status indicator badge
```

**Implementation Steps**:
1. Create progress bar component
2. Design level meter (horizontal bar)
3. Implement status badge variants
4. Layout monitor panel
5. Connect file watcher to update state
6. Bind state to UI components
7. Update every second
8. Stop button calls `commands::stop::execute()`
9. Show completion dialog
10. Return to Record panel

**Dependencies**:
- Phase 2 (Record panel)
- Phase 3 (File watcher)

**Success Criteria**:
- See live updates every second
- Progress bar animates smoothly
- Level meters update in real-time
- Can stop recording mid-way
- Completion notification shows
- Returns to Record panel

---

## Phase 5: History Service & Panel

**Status**: 📋 Planned

**Goal**: Browse and manage previous recordings

**Deliverables**:
- [ ] History service (scan recordings dir)
- [ ] Recording list with metadata
- [ ] Search functionality
- [ ] Sort options (date, size, duration)
- [ ] Filter by format
- [ ] Action buttons (Play, Show, Delete)
- [ ] Pagination
- [ ] Delete confirmation dialog

**Estimated Effort**: 7-10 days

**Key Files to Create**:
```
src/gui/
├── services/
│   └── history_service.rs  # Scan and manage recordings
└── components/
    ├── history_panel.rs    # Main history view
    ├── recording_card.rs   # Individual recording display
    ├── search_bar.rs       # Search input
    ├── filter_bar.rs       # Sort/filter controls
    └── dialog.rs           # Confirmation dialogs
```

**Implementation Steps**:
1. Create `HistoryService` to scan directory
2. Parse filenames and metadata
3. Implement search/filter/sort logic
4. Create `RecordingCard` component
5. Layout grid of recordings
6. Add search bar at top
7. Implement filter dropdowns
8. Wire up action buttons:
   - Play: open with system player
   - Show: open file location
   - Delete: show confirmation, then delete
9. Add pagination (50 per page)
10. Auto-refresh on file changes

**Dependencies**:
- Phase 3 (File watcher - for auto-refresh)

**Success Criteria**:
- Can see all recordings
- Search works instantly
- Sort/filter work correctly
- Can play/delete recordings
- Pagination works for 100+ files
- Auto-refreshes when new recording added

---

## Phase 6: Recovery Panel

**Status**: 📋 Planned

**Goal**: Recover interrupted recordings

**Deliverables**:
- [ ] Scan for incomplete recordings
- [ ] Display incomplete sessions
- [ ] Show which files exist
- [ ] Format selection per session
- [ ] Recover button per session
- [ ] Recover All buttons
- [ ] Progress modal during recovery
- [ ] Success/error notifications

**Estimated Effort**: 4-5 days

**Key Files to Create**:
```
src/gui/components/
├── recovery_panel.rs       # Main recovery view
├── incomplete_card.rs      # Incomplete recording display
├── progress_modal.rs       # Recovery progress dialog
└── notification.rs         # Toast notification
```

**Implementation Steps**:
1. Create scan logic (find `*_loopback.wav`, `*_mic.wav`)
2. Group by session ID
3. Display as cards
4. Add format radio buttons
5. Implement recover button per session
6. Show progress modal
7. Call `commands::recover::execute()`
8. Show success notification
9. Refresh list
10. Handle errors

**Dependencies**:
- Core library (`commands::recover`)

**Success Criteria**:
- Finds all incomplete recordings
- Shows what files exist
- Can recover individual sessions
- Can recover all at once
- Shows progress during recovery
- Notifications appear
- List refreshes after recovery

---

## Phase 7: Settings Panel

**Status**: 📋 Planned

**Goal**: Configure application settings

**Deliverables**:
- [ ] Directory path inputs with file picker
- [ ] Default recording settings
- [ ] Advanced options
- [ ] Theme selector
- [ ] Save/Reset buttons
- [ ] Settings persistence (JSON file)

**Estimated Effort**: 4-5 days

**Key Files to Create**:
```
src/gui/
├── components/
│   ├── settings_panel.rs   # Main settings view
│   ├── file_picker.rs      # Directory picker button
│   ├── checkbox.rs         # Checkbox component
│   └── section.rs          # Collapsible section
└── services/
    └── settings_service.rs # Save/load settings
```

**Implementation Steps**:
1. Create settings layout with sections
2. Add all input fields
3. Bind to state
4. Implement file picker (native dialog)
5. Create settings service
6. Save to `config.json`
7. Load on startup
8. Reset to defaults button
9. Validate inputs
10. Show save confirmation

**Dependencies**: None

**Success Criteria**:
- All settings visible and editable
- File picker opens native dialog
- Settings persist between sessions
- Reset works correctly
- Invalid inputs show errors

---

## Phase 8: Polish & Enhancements

**Status**: 📋 Planned

**Goal**: Add finishing touches and quality of life features

**Deliverables**:
- [ ] Keyboard shortcuts
- [ ] System notifications
- [ ] Better error messages
- [ ] Loading states
- [ ] Empty states
- [ ] Tooltips
- [ ] Animations
- [ ] Icons
- [ ] Window state persistence

**Estimated Effort**: 5-7 days

**Implementation Steps**:
1. Add keyboard shortcuts (Ctrl+R, etc.)
2. Implement system notifications
3. Add loading spinners
4. Create empty state messages
5. Add tooltips to buttons
6. Smooth animations (progress bar, transitions)
7. Add icons to buttons
8. Remember window size/position
9. Improve error dialogs
10. Add help tooltips

**Dependencies**: All previous phases

**Success Criteria**:
- Keyboard shortcuts work
- Notifications appear on desktop
- UI feels responsive
- Empty states look good
- Animations are smooth

---

## Phase 9: System Tray Integration (Optional)

**Status**: 📋 Planned

**Goal**: Minimize to system tray and show recording status

**Deliverables**:
- [ ] System tray icon
- [ ] Tray menu (Show, Record, Stop, Exit)
- [ ] Recording status in tray
- [ ] Minimize to tray
- [ ] Restore from tray
- [ ] Tray icon changes when recording

**Estimated Effort**: 3-4 days

**Dependencies**:
- `tray-icon` crate (new)

**Implementation Steps**:
1. Add `tray-icon` dependency
2. Create tray manager
3. Set up tray menu
4. Handle menu clicks
5. Update icon based on state
6. Minimize to tray on close
7. Restore on click
8. Show balloon notifications

**Success Criteria**:
- Icon appears in system tray
- Menu works correctly
- Can start/stop from tray
- Icon shows recording status
- Minimizes properly

---

## Phase 10: Cross-Platform Support

**Status**: 📋 Future

**Goal**: Support macOS and Linux

**Deliverables**:
- [ ] macOS build
- [ ] Linux build
- [ ] Platform-specific adjustments
- [ ] Native look and feel per platform

**Estimated Effort**: 10-15 days

**Dependencies**: All previous phases

**Notes**:
- GPUI is designed for cross-platform
- May need platform-specific audio handling
- Testing on each platform required

---

## Milestone Summary

### Milestone 1: MVP (Phases 1-4)
**Goal**: Basic working GUI with recording and monitoring
**Duration**: 3-4 weeks
**Features**:
- Navigate between panels
- Start recordings
- Monitor live status
- Stop recordings

### Milestone 2: Complete Feature Set (Phases 5-7)
**Goal**: Full CLI feature parity
**Duration**: 4-5 weeks
**Features**:
- Browse history
- Recover recordings
- Configure settings

### Milestone 3: Production Ready (Phases 8-9)
**Goal**: Polished user experience
**Duration**: 2-3 weeks
**Features**:
- Keyboard shortcuts
- System tray
- Notifications
- Better UX

### Milestone 4: Cross-Platform (Phase 10)
**Goal**: Support all major platforms
**Duration**: 2-3 weeks
**Features**:
- macOS support
- Linux support

---

## Development Best Practices

### For Each Phase:

1. **Plan**:
   - Review phase requirements
   - Identify components needed
   - Check dependencies

2. **Implement**:
   - Create components bottom-up
   - Test each component in isolation
   - Integrate with app state
   - Wire up events/actions

3. **Test**:
   - Manual testing of all features
   - Test edge cases
   - Test with real recordings
   - Performance check

4. **Document**:
   - Update component documentation
   - Add usage examples
   - Screenshot major features

5. **Commit**:
   - Small, focused commits
   - Clear commit messages
   - One phase per PR

### Code Quality:

- Keep components small (<200 lines)
- Extract reusable UI patterns
- Handle errors gracefully
- Show user-friendly messages
- Add logging for debugging
- Comment complex logic

### Testing Strategy:

- **Unit tests**: State management logic
- **Integration tests**: Service layer
- **Manual tests**: GUI interactions
- **Performance tests**: Large recording lists
- **Stress tests**: Multiple recordings

---

## Getting Started with Phase 1

Ready to start? Here's how to begin Phase 1:

```bash
# 1. Create theme system
touch src/gui/theme.rs

# 2. Create component stubs
mkdir -p src/gui/components
touch src/gui/components/sidebar.rs
touch src/gui/components/panel_container.rs
touch src/gui/components/nav_button.rs

# 3. Update mod.rs
# Add exports in src/gui/components/mod.rs

# 4. Update app.rs
# Use new components in main app

# 5. Test frequently
cargo run --bin audio-recorder-gui --features gui
```

See [GUI_PLAN.md](GUI_PLAN.md) for detailed component specifications.

---

## Resources

- **GPUI Docs**: https://www.gpui.rs/
- **GPUI Examples**: https://github.com/zed-industries/zed/tree/main/crates/gpui/examples
- **Zed Source**: https://github.com/zed-industries/zed (for real-world GPUI usage)
- **GUI Plan**: [GUI_PLAN.md](GUI_PLAN.md) (detailed specifications)
- **Development Guide**: [GUI_DEVELOPMENT.md](GUI_DEVELOPMENT.md)

---

## Success Metrics

### Phase Completion:
- All deliverables implemented
- Manual testing passed
- No critical bugs
- Documentation updated

### Milestone Completion:
- All phases in milestone complete
- Integration testing passed
- Performance acceptable
- Ready for user feedback

### Release Criteria:
- All milestones complete
- Cross-platform tested
- Documentation complete
- User guide written
- Known issues documented

---

## Notes

- **GPUI is experimental**: API may change, follow Zed updates
- **Start small**: Don't skip phases, build incrementally
- **Test often**: GUI bugs are hard to debug, test as you go
- **Ask for help**: GPUI community is active, use Zed Discord
- **Be patient**: First GUI implementation always takes longer than expected

Good luck! 🚀
