// Settings panel component for configuring the application

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{button::*, *};
use audio_recorder_manager::{RecorderConfig, AudioFormat};
use crate::app::QualityPreset;

pub struct SettingsPanelProps {
    pub config: RecorderConfig,
    pub default_duration: String,
    pub default_format: AudioFormat,
    pub default_quality: QualityPreset,
    pub max_manual_duration: String,
}

pub fn render_settings_panel(
    props: &SettingsPanelProps,
    _window: &mut Window,
    cx: &mut Context<crate::app::AudioRecorderApp>,
) -> Div
{
    let duration_value = props.default_duration.clone();
    let max_duration_value = props.max_manual_duration.clone();
    let format = props.default_format;
    let quality = props.default_quality;

    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(
            div()
                .text_2xl()
                .font_bold()
                .text_color(rgb(0x1a1a1a))
                .child("Settings")
        )
        // Directories Section (Read-only)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe0e0e0))
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_lg()
                        .child("📁 Directories")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(140.0)).text_sm().child("Recordings:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child(props.config.recordings_dir.display().to_string())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(140.0)).text_sm().child("Status:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child(props.config.status_dir.display().to_string())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(140.0)).text_sm().child("Signals:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child(props.config.signals_dir.display().to_string())
                        )
                )
        )
        // Recording Defaults Section (Editable)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe0e0e0))
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_lg()
                        .child("🎵 Recording Defaults")
                )
                // Default Duration (read-only for now)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(160.0)).text_sm().child("Default Duration (sec):"))
                        .child(
                            div()
                                .w(px(100.0))
                                .p_2()
                                .border_1()
                                .border_color(rgb(0xcccccc))
                                .rounded_md()
                                .bg(rgb(0xf9f9f9))
                                .text_sm()
                                .child(duration_value.clone())
                        )
                        .child(div().text_xs().text_color(rgb(0x888888)).child("(e.g., 30, 60, 300)"))
                )
                // Format Selection
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(160.0)).text_sm().child("Default Format:"))
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    Button::new("format_wav")
                                        .label("WAV")
                                        .when(format == AudioFormat::Wav, |btn| btn.primary())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_format_change(cx, AudioFormat::Wav);
                                        }))
                                )
                                .child(
                                    Button::new("format_m4a")
                                        .label("M4A")
                                        .when(format == AudioFormat::M4a, |btn| btn.primary())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_format_change(cx, AudioFormat::M4a);
                                        }))
                                )
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x888888))
                                .child(match format {
                                    AudioFormat::Wav => "(Uncompressed, larger files)",
                                    AudioFormat::M4a => "(Compressed, smaller files)",
                                })
                        )
                )
                // Quality Selection
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(160.0)).text_sm().child("Default Quality:"))
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    Button::new("quality_pro")
                                        .label("Professional")
                                        .when(quality == QualityPreset::Professional, |btn| btn.primary())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_quality_change(cx, QualityPreset::Professional);
                                        }))
                                )
                                .child(
                                    Button::new("quality_std")
                                        .label("Standard")
                                        .when(quality == QualityPreset::Standard, |btn| btn.primary())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_quality_change(cx, QualityPreset::Standard);
                                        }))
                                )
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x888888))
                                .child(match quality {
                                    QualityPreset::Professional => "(48kHz Stereo, ~11 MB/min)",
                                    QualityPreset::Standard => "(44.1kHz Stereo, ~10 MB/min)",
                                })
                        )
                )
                // Max Manual Duration (read-only for now)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(div().w(px(160.0)).text_sm().child("Max Manual Duration (sec):"))
                        .child(
                            div()
                                .w(px(100.0))
                                .p_2()
                                .border_1()
                                .border_color(rgb(0xcccccc))
                                .rounded_md()
                                .bg(rgb(0xf9f9f9))
                                .text_sm()
                                .child(max_duration_value.clone())
                        )
                        .child(div().text_xs().text_color(rgb(0x888888)).child("(2 hours = 7200 seconds)"))
                )
        )
        // Action Buttons
        .child(
            div()
                .flex()
                .gap_4()
                .child(
                    Button::new("save_settings")
                        .primary()
                        .label("💾 Save Settings")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.handle_save_settings(cx);
                        }))
                )
                .child(
                    Button::new("reset_settings")
                        .label("🔄 Reset to Defaults")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.handle_reset_settings(cx);
                        }))
                )
        )
        // Info Section
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .bg(rgb(0xf0f7ff))
                .border_1()
                .border_color(rgb(0xb3d9ff))
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_color(rgb(0x0066cc))
                        .child("ℹ️ Settings Information")
                )
                .child(div().text_sm().child("• Default Duration: Used for quick recordings in the Record panel"))
                .child(div().text_sm().child("• Format: WAV is lossless, M4A uses AAC compression"))
                .child(div().text_sm().child("• Quality: Professional (48kHz) recommended for meetings"))
                .child(div().text_sm().child("• Max Manual Duration: Safety limit for manual recordings"))
        )
}
