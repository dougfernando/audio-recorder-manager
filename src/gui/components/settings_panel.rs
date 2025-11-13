// Settings panel component for configuring the application

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{button::*, input::*, ActiveTheme, *};
use audio_recorder_manager::{RecorderConfig, AudioFormat};
use crate::app::QualityPreset;

use super::header::{SPACING_SM, SPACING_MD, SPACING_LG};

pub struct SettingsPanelProps {
    pub config: RecorderConfig,
    pub default_duration: String,
    pub default_format: AudioFormat,
    pub default_quality: QualityPreset,
    pub max_manual_duration: String,
    pub duration_input: Entity<InputState>,
    pub max_duration_input: Entity<InputState>,
}

pub fn render_settings_panel(
    props: &SettingsPanelProps,
    _window: &mut Window,
    cx: &mut Context<crate::app::AudioRecorderApp>,
) -> Div
{
    let format = props.default_format;
    let quality = props.default_quality;

    div()
        .flex()
        .flex_col()
        .gap(px(SPACING_MD))
        .p(px(SPACING_LG))
        .child(
            div()
                .text_2xl()
                .font_bold()
                .text_color(cx.theme().foreground)
                .child("Settings")
        )
        // Directories Section (Read-only)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_lg()
                        .text_color(cx.theme().foreground)
                        .child("Directories")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(140.0)).text_sm().child("Recordings:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(props.config.recordings_dir.display().to_string())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(140.0)).text_sm().child("Status:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(props.config.status_dir.display().to_string())
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(140.0)).text_sm().child("Signals:"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(props.config.signals_dir.display().to_string())
                        )
                )
        )
        // Recording Defaults Section (Editable)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM + 4.0))
                .p(px(SPACING_MD))
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_lg()
                        .text_color(cx.theme().foreground)
                        .child("Recording Defaults")
                )
                // Default Duration - now using Input component
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(160.0)).text_sm().child("Default Duration (sec):"))
                        .child(
                            div()
                                .w(px(100.0))
                                .child(Input::new(&props.duration_input))
                        )
                        .child(div().text_xs().text_color(cx.theme().muted_foreground).child("(e.g., 30, 60, 300)"))
                )
                // Format Selection
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(160.0)).text_sm().child("Default Format:"))
                        .child(
                            div()
                                .flex()
                                .gap(px(SPACING_SM))
                                .child(
                                    Button::new("format_wav")
                                        .label("WAV")
                                        .compact()
                                        .when(format == AudioFormat::Wav, |btn| btn.primary())
                                        .when(format != AudioFormat::Wav, |btn| btn.ghost())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_format_change(cx, AudioFormat::Wav);
                                        }))
                                )
                                .child(
                                    Button::new("format_m4a")
                                        .label("M4A")
                                        .compact()
                                        .when(format == AudioFormat::M4a, |btn| btn.primary())
                                        .when(format != AudioFormat::M4a, |btn| btn.ghost())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_format_change(cx, AudioFormat::M4a);
                                        }))
                                )
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
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
                        .gap(px(SPACING_MD))
                        .child(div().w(px(160.0)).text_sm().child("Default Quality:"))
                        .child(
                            div()
                                .flex()
                                .gap(px(SPACING_SM))
                                .child(
                                    Button::new("quality_pro")
                                        .label("Professional")
                                        .compact()
                                        .when(quality == QualityPreset::Professional, |btn| btn.primary())
                                        .when(quality != QualityPreset::Professional, |btn| btn.ghost())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_quality_change(cx, QualityPreset::Professional);
                                        }))
                                )
                                .child(
                                    Button::new("quality_std")
                                        .label("Standard")
                                        .compact()
                                        .when(quality == QualityPreset::Standard, |btn| btn.primary())
                                        .when(quality != QualityPreset::Standard, |btn| btn.ghost())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.handle_quality_change(cx, QualityPreset::Standard);
                                        }))
                                )
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(match quality {
                                    QualityPreset::Professional => "(48kHz Stereo, ~11 MB/min)",
                                    QualityPreset::Standard => "(44.1kHz Stereo, ~10 MB/min)",
                                })
                        )
                )
                // Max Manual Duration - now using Input component
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(div().w(px(160.0)).text_sm().child("Max Manual Duration (sec):"))
                        .child(
                            div()
                                .w(px(100.0))
                                .child(Input::new(&props.max_duration_input))
                        )
                        .child(div().text_xs().text_color(cx.theme().muted_foreground).child("(2 hours = 7200 seconds)"))
                )
        )
        // Action Buttons
        .child(
            div()
                .flex()
                .gap(px(SPACING_MD))
                .child(
                    Button::new("save_settings")
                        .primary()
                        .label("Save Settings")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.handle_save_settings(window, cx);
                        }))
                )
                .child(
                    Button::new("reset_settings")
                        .ghost()
                        .label("Reset to Defaults")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.handle_reset_settings(window, cx);
                        }))
                )
        )
        // Info Section
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(cx.theme().accent.opacity(0.1))
                .border_1()
                .border_color(cx.theme().accent.opacity(0.3))
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_color(cx.theme().accent_foreground)
                        .child("Settings Information")
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Default Duration: Used for quick recordings in the Record panel"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Format: WAV is lossless, M4A uses AAC compression"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Quality: Professional (48kHz) recommended for meetings"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Max Manual Duration: Safety limit for manual recordings"))
        )
}
