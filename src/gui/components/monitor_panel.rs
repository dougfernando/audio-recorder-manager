// Monitor panel component for viewing recording progress

use gpui::*;
use gpui_component::{button::*, *};
use crate::app::AudioRecorderApp;

pub fn render_monitor_panel(
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
                .child("Recording in Progress")
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
                        .font_semibold()
                        .text_color(rgb(0x333333))
                        .child("Session Information")
                )
                .child(div().text_sm().child("Session ID: rec-20250111_153045"))
                .child(div().text_sm().child("File: recording_20250111_153045.wav"))
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
                        .child("Progress: 67%")
                )
                .child(div().text_sm().child("Time: 20s / 30s"))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .bg(rgb(0xf0fff4))
                .border_1()
                .border_color(rgb(0xb3e6c7))
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().font_semibold().child("System Audio (Loopback)"))
                        .child(div().text_color(rgb(0x00aa00)).child("🔊 Audio Detected"))
                )
                .child(div().text_sm().child("1,440 frames | Sample Rate: 48,000 Hz"))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .bg(rgb(0xf0fff4))
                .border_1()
                .border_color(rgb(0xb3e6c7))
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().font_semibold().child("Microphone"))
                        .child(div().text_color(rgb(0x00aa00)).child("🎤 Audio Detected"))
                )
                .child(div().text_sm().child("1,425 frames | Sample Rate: 48,000 Hz (matched)"))
        )
        .child(
            Button::new("stop_recording")
                .label("⏹️ STOP RECORDING")
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.stop_recording(cx);
                }))
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x666666))
                .italic()
                .child("💡 The recording is happening in the background. You can minimize this window.")
        )
}
