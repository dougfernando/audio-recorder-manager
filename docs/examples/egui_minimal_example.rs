// Example: Minimal GUI with egui (immediate mode)
// This demonstrates how to implement the GUI using egui instead of GPUI
//
// To use this:
// 1. Update Cargo.toml dependencies:
//    egui = { version = "0.24", optional = true }
//    eframe = { version = "0.24", optional = true }
//    [features]
//    gui = ["egui", "eframe", "notify", "notify-debouncer-full"]
//
// 2. Replace src/gui/main.rs with this content (adapted)
//
// 3. Build: cargo run --bin audio-recorder-gui --features gui

use eframe::egui;
use audio_recorder_manager::{RecorderConfig, AudioFormat, RecordingDuration, RecordingQuality};

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Audio Recorder Manager",
        options,
        Box::new(|_cc| Box::new(AudioRecorderApp::new())),
    )
}

struct AudioRecorderApp {
    config: RecorderConfig,
    active_panel: ActivePanel,

    // Record panel state
    duration: String,
    is_manual: bool,
    format: AudioFormat,
    quality: RecordingQuality,

    // Status
    status_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActivePanel {
    Record,
    Monitor,
    History,
    Recovery,
    Settings,
}

impl AudioRecorderApp {
    fn new() -> Self {
        Self {
            config: RecorderConfig::new(),
            active_panel: ActivePanel::Record,
            duration: "30".to_string(),
            is_manual: false,
            format: AudioFormat::Wav,
            quality: RecordingQuality::professional(),
            status_message: String::new(),
        }
    }
}

impl eframe::App for AudioRecorderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎙️ Audio Recorder Manager");
                ui.separator();
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
        });

        // Sidebar
        egui::SidePanel::left("sidebar").min_width(200.0).show(ctx, |ui| {
            ui.heading("Navigation");
            ui.separator();

            if ui.selectable_label(self.active_panel == ActivePanel::Record, "🔴 Record").clicked() {
                self.active_panel = ActivePanel::Record;
            }
            if ui.selectable_label(self.active_panel == ActivePanel::Monitor, "📊 Monitor").clicked() {
                self.active_panel = ActivePanel::Monitor;
            }
            if ui.selectable_label(self.active_panel == ActivePanel::History, "📁 History").clicked() {
                self.active_panel = ActivePanel::History;
            }
            if ui.selectable_label(self.active_panel == ActivePanel::Recovery, "🔧 Recovery").clicked() {
                self.active_panel = ActivePanel::Recovery;
            }
            if ui.selectable_label(self.active_panel == ActivePanel::Settings, "⚙️ Settings").clicked() {
                self.active_panel = ActivePanel::Settings;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.separator();
                ui.small("Built with Rust + egui");
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_panel {
                ActivePanel::Record => self.render_record_panel(ui),
                ActivePanel::Monitor => self.render_monitor_panel(ui),
                ActivePanel::History => self.render_history_panel(ui),
                ActivePanel::Recovery => self.render_recovery_panel(ui),
                ActivePanel::Settings => self.render_settings_panel(ui),
            }
        });

        // Status bar
        if !self.status_message.is_empty() {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);
                });
            });
        }
    }
}

impl AudioRecorderApp {
    fn render_record_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Start New Recording");
        ui.add_space(10.0);

        egui::Grid::new("record_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // Duration
                ui.label("Duration:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.duration);
                    ui.label("seconds");
                    ui.checkbox(&mut self.is_manual, "Manual mode (-1)");
                });
                ui.end_row();

                // Format
                ui.label("Format:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.format, AudioFormat::Wav, "WAV");
                    ui.radio_value(&mut self.format, AudioFormat::M4a, "M4A");
                });
                ui.end_row();

                // Quality
                ui.label("Quality:");
                egui::ComboBox::from_id_source("quality")
                    .selected_text(&self.quality.name)
                    .show_ui(ui, |ui| {
                        let qualities = vec![
                            RecordingQuality::quick(),
                            RecordingQuality::standard(),
                            RecordingQuality::professional(),
                            RecordingQuality::high(),
                        ];
                        for q in qualities {
                            ui.selectable_value(&mut self.quality, q.clone(), &q.name);
                        }
                    });
                ui.end_row();

                // Device
                ui.label("Device:");
                ui.label("System Audio + Microphone (Dual-channel)");
                ui.end_row();
            });

        ui.add_space(20.0);

        // Preview
        ui.group(|ui| {
            ui.label("Preview:");
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let preview = format!("recording_{}.{}", timestamp, self.format.extension());
            ui.monospace(&preview);
            ui.label(format!("~{} per minute", self.quality.size_per_min));
        });

        ui.add_space(20.0);

        // Start button
        if ui.add_sized([200.0, 40.0], egui::Button::new("🔴 START RECORDING")).clicked() {
            self.start_recording();
        }
    }

    fn render_monitor_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recording in Progress");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Session: rec-20250111_153045");
            ui.label("File: recording_20250111_153045.wav");
        });

        ui.add_space(10.0);

        // Progress
        ui.group(|ui| {
            ui.label("Progress:");
            ui.add(egui::ProgressBar::new(0.67).text("67%"));
            ui.label("Time: 20s / 30s");
        });

        ui.add_space(10.0);

        // Audio levels
        ui.group(|ui| {
            ui.label("System Audio (Loopback)");
            ui.add(egui::ProgressBar::new(0.8).text("1,440 frames"));
            ui.label("🔊 Audio Detected");
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Microphone");
            ui.add(egui::ProgressBar::new(0.75).text("1,425 frames"));
            ui.label("🎤 Audio Detected");
        });

        ui.add_space(20.0);

        if ui.add_sized([200.0, 40.0], egui::Button::new("⏹️ STOP RECORDING")).clicked() {
            self.stop_recording();
        }
    }

    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recording History");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut String::new());
            ui.label("Sort:");
            egui::ComboBox::from_id_source("sort")
                .selected_text("Date (newest first)")
                .show_ui(ui, |ui| {
                    ui.label("Date");
                    ui.label("Size");
                    ui.label("Duration");
                });
        });

        ui.separator();

        ui.label("No recordings found. Start recording to see them here!");
    }

    fn render_recovery_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recovery Center");
        ui.add_space(10.0);

        ui.label("Incomplete Recordings Found: 0");
        ui.add_space(10.0);

        ui.label("No incomplete recordings found. Great! All your recordings completed successfully.");
    }

    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(10.0);

        ui.collapsing("Directories", |ui| {
            ui.horizontal(|ui| {
                ui.label("Recordings:");
                ui.text_edit_singleline(&mut self.config.recordings_dir.to_string_lossy().to_string());
                if ui.button("📁").clicked() {
                    // Open file picker
                }
            });

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.text_edit_singleline(&mut self.config.status_dir.to_string_lossy().to_string());
                if ui.button("📁").clicked() {
                    // Open file picker
                }
            });
        });

        ui.collapsing("Recording Defaults", |ui| {
            ui.horizontal(|ui| {
                ui.label("Default Duration:");
                ui.text_edit_singleline(&mut String::from("30"));
                ui.label("seconds");
            });
        });

        ui.collapsing("Appearance", |ui| {
            ui.horizontal(|ui| {
                ui.label("Theme:");
                ui.radio_value(&mut 0, 0, "Light");
                ui.radio_value(&mut 0, 1, "Dark");
                ui.radio_value(&mut 0, 2, "System");
            });
        });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if ui.button("💾 Save Settings").clicked() {
                self.status_message = "Settings saved!".to_string();
            }
            if ui.button("🔄 Reset to Defaults").clicked() {
                self.status_message = "Settings reset to defaults".to_string();
            }
        });
    }

    fn start_recording(&mut self) {
        self.status_message = "Starting recording...".to_string();
        self.active_panel = ActivePanel::Monitor;

        // In real implementation:
        // tokio::spawn(async move {
        //     commands::record::execute(duration, format, quality, config).await
        // });
    }

    fn stop_recording(&mut self) {
        self.status_message = "Stopping recording...".to_string();
        self.active_panel = ActivePanel::Record;

        // In real implementation:
        // tokio::spawn(async move {
        //     commands::stop::execute(None, config).await
        // });
    }
}
