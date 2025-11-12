// Main application state and UI implementation using GPUI

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{button::*, input::{InputEvent, InputState}};
use audio_recorder_manager::{RecorderConfig, AudioFormat, RecordingDuration, RecordingQuality};
use std::sync::Arc;

use super::state::{AppState, ActivePanel};
use super::services::RecorderService;
use super::components::*;

pub struct AudioRecorderApp {
    state: AppState,
    duration_text: String,
    duration_input: Entity<InputState>,
    status_message: String,
    recorder_service: Arc<RecorderService>,
    tokio_runtime: Arc<tokio::runtime::Runtime>,
    // Settings fields
    settings_default_duration: String,
    settings_default_format: AudioFormat,
    settings_default_quality: QualityPreset,
    settings_max_manual_duration: String,
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

        // Create input state for duration field
        let duration_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Duration (seconds)")
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

        Self {
            state,
            duration_text: "30".to_string(),
            duration_input,
            status_message: String::new(),
            recorder_service,
            tokio_runtime,
            settings_default_duration: "30".to_string(),
            settings_default_format: AudioFormat::Wav,
            settings_default_quality: QualityPreset::Professional,
            settings_max_manual_duration: "7200".to_string(),
        }
    }

    pub fn start_recording(&mut self, cx: &mut Context<Self>) {
        // Parse duration from text input
        let duration_secs: i64 = self.duration_text.parse().unwrap_or(30);
        let duration = if duration_secs < 0 {
            RecordingDuration::Manual { max: 7200 } // 2 hours max
        } else {
            RecordingDuration::Fixed(duration_secs as u64)
        };

        // Get format and quality from state (or use defaults)
        let format = self
            .state
            .recording_state
            .as_ref()
            .map(|rs| rs.format)
            .unwrap_or(AudioFormat::Wav);
        let quality = self
            .state
            .recording_state
            .as_ref()
            .map(|rs| rs.quality.clone())
            .unwrap_or_else(|| RecordingQuality::professional());

        // Start recording via service
        let recorder_service = self.recorder_service.clone();
        cx.spawn(async move |this, cx| {
            match recorder_service
                .start_recording(duration, format, quality)
                .await
            {
                Ok(session_id) => {
                    this.update(cx, |this, cx| {
                        this.status_message = format!("Recording started: {}", session_id);
                        this.state.active_panel = ActivePanel::Monitor;
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    this.update(cx, |this, cx| {
                        this.status_message = format!("Failed to start recording: {}", e);
                        cx.notify();
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        let recorder_service = self.recorder_service.clone();

        cx.spawn(async move |this, cx| {
            match recorder_service.stop_recording().await {
                Ok(_) => {
                    this.update(cx, |this, cx| {
                        this.status_message = "Recording stopped successfully".to_string();
                        this.state.active_panel = ActivePanel::Record;
                        cx.notify();
                    }).ok();
                }
                Err(e) => {
                    this.update(cx, |this, cx| {
                        this.status_message = format!("Failed to stop recording: {}", e);
                        cx.notify();
                    }).ok();
                }
            }
        })
        .detach();
    }

    pub fn handle_panel_change(&mut self, cx: &mut Context<Self>, panel: ActivePanel) {
        self.state.active_panel = panel;
        cx.notify();
    }

    pub fn handle_scan_recovery(&mut self, cx: &mut Context<Self>) {
        self.status_message = "Scanning for incomplete recordings...".to_string();
        cx.notify();
    }

    pub fn handle_save_settings(&mut self, cx: &mut Context<Self>) {
        // Parse and validate settings
        let _default_duration: u64 = self.settings_default_duration.parse().unwrap_or(30);
        let max_manual_duration: u64 = self.settings_max_manual_duration.parse().unwrap_or(7200);

        // Update the config (in a real app, this would persist to disk)
        self.state.config.max_manual_duration_secs = max_manual_duration;

        self.status_message = "Settings saved successfully!".to_string();
        cx.notify();
    }

    pub fn handle_reset_settings(&mut self, cx: &mut Context<Self>) {
        self.status_message = "Settings reset to defaults".to_string();
        self.state.config = RecorderConfig::new();
        self.settings_default_duration = "30".to_string();
        self.settings_default_format = AudioFormat::Wav;
        self.settings_default_quality = QualityPreset::Professional;
        self.settings_max_manual_duration = "7200".to_string();
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
        let duration = self.duration_text.clone();
        let status = self.status_message.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xfafafa))
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
                                        duration_text: duration,
                                        duration_input: self.duration_input.clone(),
                                    },
                                    window,
                                    cx
                                ),
                                ActivePanel::Monitor => render_monitor_panel(
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
                                        default_duration: self.settings_default_duration.clone(),
                                        default_format: self.settings_default_format,
                                        default_quality: self.settings_default_quality,
                                        max_manual_duration: self.settings_max_manual_duration.clone(),
                                    },
                                    window,
                                    cx
                                ),
                            })
                    )
            )
            .when(!status.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .h(px(40.0))
                        .px_6()
                        .bg(rgb(0x0066cc))
                        .text_color(rgb(0xffffff))
                        .child(status.clone())
                        .child(
                            Button::new("close_status")
                                .label("✖")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.status_message.clear();
                                    cx.notify();
                                }))
                        )
                )
            })
    }
}
