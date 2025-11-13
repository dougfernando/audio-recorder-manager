// Header component for the main application window

use gpui::*;
use gpui_component::{ActiveTheme, badge::Badge, Icon, IconName};

// Spacing constants
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 16.0;
pub const SPACING_LG: f32 = 24.0;

pub fn render_header<V>(_window: &mut Window, cx: &mut Context<V>) -> Div
where
    V: 'static,
{
    div()
        .flex()
        .items_center()
        .h(px(60.0))
        .px(px(SPACING_LG))
        .bg(cx.theme().background)
        .border_b_2()
        .border_color(cx.theme().border)
        .shadow_lg()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACING_SM))
                .child(
                    Icon::new(IconName::Mic)
                        .large()
                        .text_color(cx.theme().primary)
                )
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(cx.theme().foreground)
                        .child("Audio Recorder Manager")
                )
        )
        .child(
            div()
                .ml_auto()
                .child(
                    Badge::new()
                        .small()
                        .color(cx.theme().accent)
                        .child(
                            div()
                                .px(px(SPACING_SM))
                                .py(px(4.0))
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().accent_foreground)
                                .child(format!("v{}", env!("CARGO_PKG_VERSION")))
                        )
                )
        )
}
