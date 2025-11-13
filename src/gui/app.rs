// Main application state and UI implementation using GPUI

use gpui::*;
use gpui_component::{
    input::{InputEvent, InputState},
    notification::NotificationType,
    ActiveTheme,
    Root,
    WindowExt,
};
use audio_recorder_manager::{RecorderConfig, AudioFormat, RecordingDuration, RecordingQuality};
use std::sync::Arc;

use super::state::{AppState, ActivePanel, RecordingState};
use super::services::RecorderService;
use super::components::*;

pub struct AudioRecorderApp {
    state: AppState,
    duration_text: String,
    duration_input: Entity<InputState>,
    recorder_service: Arc<RecorderService>,
    // Keep runtime alive for the duration of the app
    _tokio_runtime: Arc<tokio::runtime::Runtime>,
    // Settings fields
    settings_default_duration: String,
    settings_default_format: AudioFormat,
    settings_default_quality: QualityPreset,
    settings_max_manual_duration: String,
    // Settings input states
    settings_duration_input: Entity<InputState>,
    settings_max_duration_input: Entity<InputState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    Professional,
    Standard,
}

impl AudioRecorderApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = RecorderConfig::new();
        let state = AppState::new(config.clone());
        let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let recorder_service = Arc::new(RecorderService::new(config, tokio_runtime.clone()));

        // Create input state for duration field with validation
        let duration_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Duration (seconds)")
                .default_value("30")
                .validate(|s, _| s.parse::<i64>().is_ok() || s == "-1")
        });

        // Subscribe to input changes
        cx.subscribe_in(&duration_input, window, |this, input_state, event, _window, cx| {
            if let InputEvent::Change = event {
                let value = input_state.read(cx).value();
                this.duration_text = value.to_string();
                cx.notify();
            }
        })
        .detach();

        // Create input states for settings panel
        let settings_duration_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Duration")
                .default_value("30")
                .validate(|s, _| s.parse::<u64>().is_ok())
        });

        let settings_max_duration_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Max Duration")
                .default_value("7200")
                .validate(|s, _| s.parse::<u64>().is_ok())
        });

        // Subscribe to settings input changes
        cx.subscribe_in(&settings_duration_input, window, |this, input_state, event, _window, cx| {
            if let InputEvent::Change = event {
                let value = input_state.read(cx).value();
                this.settings_default_duration = value.to_string();
                cx.notify();
            }
        })
        .detach();

        cx.subscribe_in(&settings_max_duration_input, window, |this, input_state, event, _window, cx| {
            if let InputEvent::Change = event {
                let value = input_state.read(cx).value();
                this.settings_max_manual_duration = value.to_string();
                cx.notify();
            }
        })
        .detach();

        Self {
            state,
            duration_text: "30".to_string(),
            duration_input,
            recorder_service,
            _tokio_runtime: tokio_runtime,
            settings_default_duration: "30".to_string(),
            settings_default_format: AudioFormat::Wav,
            settings_default_quality: QualityPreset::Professional,
            settings_max_manual_duration: "7200".to_string(),
            settings_duration_input,
            settings_max_duration_input,
        }
    }

    pub fn start_recording(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Parse duration from text input
        let duration_secs: i64 = self.duration_text.parse().unwrap_or(30);
        let duration = if duration_secs < 0 {
            RecordingDuration::Manual { max: 7200 } // 2 hours max
        } else {
            RecordingDuration::Fixed(duration_secs as u64)
        };

        // Get format and quality from defaults
        let format = self.settings_default_format;
        let quality = match self.settings_default_quality {
            QualityPreset::Professional => RecordingQuality::professional(),
            QualityPreset::Standard => RecordingQuality::standard(),
        };

        // Start recording via service
        let recorder_service = self.recorder_service.clone();
        let duration_clone = duration.clone();
        let format_clone = format.clone();
        let quality_clone = quality.clone();

        cx.spawn(async move |this, cx| {
            match recorder_service
                .start_recording(duration, format, quality)
                .await
            {
                Ok(session_id) => {
                    this.update(cx, |this, cx| {
                        // Initialize recording state
                        let filename = format!(
                            "recording_{}.{}",
                            chrono::Local::now().format("%Y%m%d_%H%M%S"),
                            format_clone.extension()
                        );

                        this.state.recording_state = Some(RecordingState {
                            session_id,
                            filename,
                            duration: duration_clone,
                            format: format_clone,
                            quality: quality_clone,
                            is_manual: matches!(duration_clone, RecordingDuration::Manual { .. }),
                            elapsed: 0,
                            progress: 0,
                            device: String::from("Unknown"),
                            sample_rate: 0,
                            channels: 0,
                            frames_captured: 0,
                            has_audio: false,
                            status: String::from("recording"),
                        });

                        // Switch to monitor panel
                        this.state.active_panel = ActivePanel::Monitor;

                        // Schedule first progress update
                        Self::schedule_recording_update(cx);

                        cx.notify();
                    })
                    .ok();
                }
                Err(_e) => {
                    // Update to show error state if needed
                    this.update(cx, |_this, cx| {
                        cx.notify();
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    pub fn stop_recording(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let recorder_service = self.recorder_service.clone();

        cx.spawn(async move |this, cx| {
            match recorder_service.stop_recording().await {
                Ok(_) => {
                    this.update(cx, |this, cx| {
                        // Clear recording state
                        this.state.recording_state = None;
                        // Note: Notifications from async context would require window access
                        // For now, the panel change provides visual feedback
                        this.state.active_panel = ActivePanel::Record;
                        cx.notify();
                    }).ok();
                }
                Err(_e) => {
                    // Update to show error state if needed
                    this.update(cx, |_this, cx| {
                        cx.notify();
                    }).ok();
                }
            }
        })
        .detach();
    }

    /// Update recording progress from status file (called periodically)
    pub fn update_recording_progress(&mut self, cx: &mut Context<Self>) {
        let recorder_service = self.recorder_service.clone();

        cx.spawn(async move |this, cx| {
            if let Ok(Some(status)) = recorder_service.get_recording_status().await {
                this.update(cx, |this, cx| {
                    // Update recording state with latest status
                    if let Some(ref mut recording_state) = this.state.recording_state {
                        recording_state.elapsed = status.elapsed;
                        recording_state.progress = status.progress;
                        recording_state.device = status.device.clone();
                        recording_state.sample_rate = status.sample_rate;
                        recording_state.channels = status.channels;
                        recording_state.frames_captured = status.frames_captured;
                        recording_state.has_audio = status.has_audio;
                        recording_state.status = status.status.clone();

                        // If recording completed, clear state
                        if status.status == "completed" || status.status == "stopped" {
                            this.state.recording_state = None;
                            this.state.active_panel = ActivePanel::Record;
                        }

                        cx.notify();
                    }
                }).ok();
            }
        })
        .detach();
    }

    /// Schedule a single recording progress update (recursively called for continuous updates)
    fn schedule_recording_update(cx: &mut Context<Self>) {
        cx.spawn(|this: WeakEntity<Self>, cx| async move {
            // Wait 500ms before checking status
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Update recording state and schedule next update if still recording
            // Both operations must happen in a SINGLE update() call because cx is consumed
            let _ = this.update(cx, |this, cx| {
                // Update progress from status file
                this.update_recording_progress(cx);

                // Schedule next update if still recording
                if this.state.recording_state.is_some() {
                    Self::schedule_recording_update(cx);
                }
            });
        })
        .detach();
    }

    pub fn handle_panel_change(&mut self, cx: &mut Context<Self>, panel: ActivePanel) {
        self.state.active_panel = panel;
        cx.notify();
    }

    pub fn handle_scan_recovery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.push_notification(
            (NotificationType::Info, "Scanning for incomplete recordings..."),
            cx
        );
        cx.notify();
    }

    pub fn handle_save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Parse and validate settings
        let _default_duration: u64 = self.settings_default_duration.parse().unwrap_or(30);
        let max_manual_duration: u64 = self.settings_max_manual_duration.parse().unwrap_or(7200);

        // Update the config (in a real app, this would persist to disk)
        self.state.config.max_manual_duration_secs = max_manual_duration;

        window.push_notification(
            (NotificationType::Success, "Settings saved successfully!"),
            cx
        );
        cx.notify();
    }

    pub fn handle_reset_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.config = RecorderConfig::new();
        self.settings_default_duration = "30".to_string();
        self.settings_default_format = AudioFormat::Wav;
        self.settings_default_quality = QualityPreset::Professional;
        self.settings_max_manual_duration = "7200".to_string();

        // Note: Input states will reflect new values on next re-render
        // as they're bound to settings_default_duration and settings_max_manual_duration

        window.push_notification(
            (NotificationType::Success, "Settings reset to defaults"),
            cx
        );
        cx.notify();
    }

    pub fn handle_duration_change(&mut self, cx: &mut Context<Self>, value: String) {
        self.settings_default_duration = value;
        cx.notify();
    }

    pub fn handle_max_duration_change(&mut self, cx: &mut Context<Self>, value: String) {
        self.settings_max_manual_duration = value;
        cx.notify();
    }

    pub fn handle_format_change(&mut self, cx: &mut Context<Self>, format: AudioFormat) {
        self.settings_default_format = format;
        cx.notify();
    }

    pub fn handle_quality_change(&mut self, cx: &mut Context<Self>, quality: QualityPreset) {
        self.settings_default_quality = quality;
        cx.notify();
    }
}

impl Render for AudioRecorderApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_panel = self.state.active_panel;
        let config = self.state.config.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .child(render_header(window, cx))
            .child(
                // Main content area
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(render_sidebar(&SidebarProps { active_panel }, window, cx))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .flex_col()
                            .child(match active_panel {
                                ActivePanel::Record => render_record_panel(
                                    &RecordPanelProps {
                                        config: config.clone(),
                                        duration_input: self.duration_input.clone(),
                                    },
                                    window,
                                    cx
                                ),
                                ActivePanel::Monitor => render_monitor_panel(
                                    &MonitorPanelProps {
                                        recording_state: self.state.recording_state.as_ref(),
                                    },
                                    window,
                                    cx
                                ),
                                ActivePanel::History => render_history_panel(
                                    window,
                                    cx
                                ),
                                ActivePanel::Recovery => render_recovery_panel(
                                    window,
                                    cx
                                ),
                                ActivePanel::Settings => render_settings_panel(
                                    &SettingsPanelProps {
                                        config,
                                        default_format: self.settings_default_format,
                                        default_quality: self.settings_default_quality,
                                        duration_input: self.settings_duration_input.clone(),
                                        max_duration_input: self.settings_max_duration_input.clone(),
                                    },
                                    window,
                                    cx
                                ),
                            })
                    )
            )
            // Add overlay layers for dialogs, sheets, and notifications
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
