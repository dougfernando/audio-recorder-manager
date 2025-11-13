// Record panel component for starting new recordings

use gpui::*;
use gpui_component::{button::*, input::*, ActiveTheme, *};
use audio_recorder_manager::RecorderConfig;
use crate::app::AudioRecorderApp;

use super::header::{SPACING_SM, SPACING_MD, SPACING_LG};

pub struct RecordPanelProps {
    pub config: RecorderConfig,
    pub duration_text: String,
    pub duration_input: Entity<InputState>,
}

pub fn render_record_panel(
    props: &RecordPanelProps,
    _window: &mut Window,
    cx: &mut Context<AudioRecorderApp>,
) -> Div
{
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
                .child("Start New Recording")
        )
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
                .shadow_sm()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child("Duration:")
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(SPACING_SM))
                                .child(
                                    div()
                                        .w(px(100.0))
                                        .child(Input::new(&props.duration_input))
                                )
                                .child("seconds")
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .ml(px(SPACING_SM))
                                        .child("(use -1 for manual)")
                                )
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child("Format:")
                        )
                        .child("WAV / M4A")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child("Quality:")
                        )
                        .child("Professional (48kHz)")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child("Device:")
                        )
                        .child("System Audio + Microphone (Dual-channel)")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACING_MD))
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child("Output:")
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(props.config.recordings_dir.display().to_string())
                        )
                )
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(cx.theme().muted.opacity(0.2))
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child("Preview:")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().accent_foreground)
                        .child(format!(
                            "recording_{}.wav",
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        ))
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Estimated size: ~10 MB per minute")
                )
        )
        .child(
            Button::new("start_recording")
                .primary()
                .label("START RECORDING")
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.start_recording(window, cx);
                }))
        )
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
                        .child("Quick Tips")
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Use Manual mode (-1 seconds) to record until you manually stop"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• WAV format is uncompressed, M4A is compressed (smaller files)"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Professional quality (48kHz) is recommended for meetings"))
                .child(div().text_sm().text_color(cx.theme().foreground).child("• Recording captures both system audio and microphone simultaneously"))
        )
}
