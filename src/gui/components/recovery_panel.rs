// Recovery panel component for recovering incomplete recordings

use gpui::*;
use gpui_component::{button::*, ActiveTheme, *};
use crate::app::AudioRecorderApp;

use super::header::{SPACING_MD, SPACING_LG};

pub fn render_recovery_panel(
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
                .child("Recovery Center")
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACING_MD))
                .p(px(SPACING_MD))
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(div().text_color(cx.theme().foreground).child("Scan for incomplete recordings..."))
                .child(
                    Button::new("scan_recovery")
                        .ghost()
                        .label("🔍 Scan Now")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.handle_scan_recovery(window, cx);
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
                .gap(px(SPACING_MD))
                .child(
                    div()
                        .text_3xl()
                        .child("✅")
                )
                .child(
                    div()
                        .text_lg()
                        .text_color(cx.theme().success)
                        .child("No incomplete recordings found")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Great! All your recordings completed successfully.")
                )
        )
}
