// Sidebar navigation component

use gpui::*;
use gpui_component::{button::*, ActiveTheme, *};
use crate::state::ActivePanel;
use crate::app::AudioRecorderApp;

use super::header::{SPACING_SM, SPACING_MD};

pub struct SidebarProps {
    pub active_panel: ActivePanel,
}

pub fn render_sidebar(
    props: &SidebarProps,
    _window: &mut Window,
    cx: &mut Context<AudioRecorderApp>,
) -> Div
{
    let active = props.active_panel;

    div()
        .flex()
        .flex_col()
        .w(px(220.0))
        .h_full()
        .bg(cx.theme().muted.opacity(0.3))
        .border_r_1()
        .border_color(cx.theme().border)
        .p(px(SPACING_MD))
        .gap(px(SPACING_SM))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .pb(px(SPACING_MD))
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_lg()
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child("Navigation")
                )
        )
        .child(
            Button::new("btn_record")
                .label("Record")
                .when(active == ActivePanel::Record, |btn| btn.primary())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.handle_panel_change(cx, ActivePanel::Record);
                }))
        )
        .child(
            Button::new("btn_monitor")
                .label("Monitor")
                .when(active == ActivePanel::Monitor, |btn| btn.primary())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.handle_panel_change(cx, ActivePanel::Monitor);
                }))
        )
        .child(
            Button::new("btn_history")
                .label("History")
                .when(active == ActivePanel::History, |btn| btn.primary())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.handle_panel_change(cx, ActivePanel::History);
                }))
        )
        .child(
            Button::new("btn_recovery")
                .label("Recovery")
                .when(active == ActivePanel::Recovery, |btn| btn.primary())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.handle_panel_change(cx, ActivePanel::Recovery);
                }))
        )
        .child(
            Button::new("btn_settings")
                .label("Settings")
                .when(active == ActivePanel::Settings, |btn| btn.primary())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.handle_panel_change(cx, ActivePanel::Settings);
                }))
        )
        .child(
            div()
                .flex()
                .flex_1()
                .items_end()
                .justify_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Built with Rust + GPUI")
                )
        )
}
