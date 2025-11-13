// History panel component for viewing past recordings

use gpui::*;
use gpui_component::{button::*, ActiveTheme, Icon, IconName};
use crate::state::ActivePanel;
use crate::app::AudioRecorderApp;

use super::header::{SPACING_MD, SPACING_LG};

pub fn render_history_panel(
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
                .child("Recording History")
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
                .child(div().text_color(cx.theme().foreground).child("Search:"))
                .child(div().text_color(cx.theme().muted_foreground).child("Sort by: Date (newest first)"))
                .child(div().text_color(cx.theme().muted_foreground).child("Format: All"))
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
                        .child("📁")
                )
                .child(
                    div()
                        .text_lg()
                        .text_color(cx.theme().muted_foreground)
                        .child("No recordings found")
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground.opacity(0.7))
                        .child("Start recording to see your recordings here!")
                )
                .child(
                    Button::new("goto_record")
                        .primary()
                        .icon(IconName::Mic)
                        .label("Go to Record Panel")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.handle_panel_change(cx, ActivePanel::Record);
                        }))
                )
        )
}
