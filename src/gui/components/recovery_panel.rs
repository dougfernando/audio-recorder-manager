// Recovery panel component for recovering incomplete recordings

use gpui::*;
use gpui_component::{button::*, *};
use crate::app::AudioRecorderApp;

pub fn render_recovery_panel(
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
                .child("Recovery Center")
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .p_4()
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe0e0e0))
                .rounded_lg()
                .child(div().child("Scan for incomplete recordings..."))
                .child(
                    Button::new("scan_recovery")
                        .label("🔄 Scan Now")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.handle_scan_recovery(cx);
                        }))
                )
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_4()
                .child(
                    div()
                        .text_3xl()
                        .child("✅")
                )
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(0x00aa00))
                        .child("No incomplete recordings found")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x666666))
                        .child("Great! All your recordings completed successfully.")
                )
        )
}
