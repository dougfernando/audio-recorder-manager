// History panel component for viewing past recordings

use gpui::*;
use gpui_component::{button::*, *};
use crate::state::ActivePanel;
use crate::app::AudioRecorderApp;

pub fn render_history_panel(
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
                .child("Recording History")
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
                .child(div().child("🔍 Search:"))
                .child(div().child("Sort by: Date (newest first)"))
                .child(div().child("Format: All"))
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
                        .child("📁")
                )
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(0x666666))
                        .child("No recordings found")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x999999))
                        .child("Start recording to see your recordings here!")
                )
                .child(
                    Button::new("goto_record")
                        .primary()
                        .label("🔴 Go to Record Panel")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.handle_panel_change(cx, ActivePanel::Record);
                        }))
                )
        )
}
