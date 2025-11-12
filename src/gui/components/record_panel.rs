// Record panel component for starting new recordings

use gpui::*;
use gpui_component::{button::*, *};
use audio_recorder_manager::RecorderConfig;
use crate::app::AudioRecorderApp;

pub struct RecordPanelProps {
    pub config: RecorderConfig,
    pub duration_text: String,
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
        .gap_4()
        .p_6()
        .child(
            div()
                .text_2xl()
                .font_bold()
                .text_color(rgb(0x1a1a1a))
                .child("Start New Recording")
        )
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
                .shadow_sm()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(rgb(0x333333))
                                .child("Duration:")
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .w(px(80.0))
                                        .p_2()
                                        .border_1()
                                        .border_color(rgb(0xcccccc))
                                        .rounded_md()
                                        .child(props.duration_text.clone())
                                )
                                .child("seconds")
                        )
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(rgb(0x333333))
                                .child("Format:")
                        )
                        .child("WAV / M4A")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(rgb(0x333333))
                                .child("Quality:")
                        )
                        .child("Professional (48kHz)")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(rgb(0x333333))
                                .child("Device:")
                        )
                        .child("System Audio + Microphone (Dual-channel)")
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .w(px(120.0))
                                .font_semibold()
                                .text_color(rgb(0x333333))
                                .child("Output:")
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child(props.config.recordings_dir.display().to_string())
                        )
                )
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .bg(rgb(0xf8f9fa))
                .border_1()
                .border_color(rgb(0xe0e0e0))
                .rounded_lg()
                .child(
                    div()
                        .font_semibold()
                        .text_color(rgb(0x333333))
                        .child("Preview:")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x0066cc))
                        .child(format!(
                            "recording_{}.wav",
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        ))
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x666666))
                        .child("Estimated size: ~10 MB per minute")
                )
        )
        .child(
            Button::new("start_recording")
                .primary()
                .label("🔴 START RECORDING")
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.start_recording(cx);
                }))
        )
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
                        .child("ℹ️ Quick Tips")
                )
                .child(div().text_sm().child("• Use Manual mode (-1 seconds) to record until you manually stop"))
                .child(div().text_sm().child("• WAV format is uncompressed, M4A is compressed (smaller files)"))
                .child(div().text_sm().child("• Professional quality (48kHz) is recommended for meetings"))
                .child(div().text_sm().child("• Recording captures both system audio and microphone simultaneously"))
        )
}
