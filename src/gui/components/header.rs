// Header component for the main application window

use gpui::*;
use gpui_component::*;

pub fn render_header<V>(_window: &mut Window, _cx: &mut Context<V>) -> Div
where
    V: 'static,
{
    div()
        .flex()
        .items_center()
        .h(px(60.0))
        .px_6()
        .bg(rgb(0xffffff))
        .border_b_1()
        .border_color(rgb(0xe0e0e0))
        .shadow_sm()
        .child(
            div()
                .text_xl()
                .font_bold()
                .text_color(rgb(0x1a1a1a))
                .child("🎙️ Audio Recorder Manager")
        )
        .child(
            div()
                .ml_4()
                .text_xs()
                .text_color(rgb(0x999999))
                .child(format!("v{}", env!("CARGO_PKG_VERSION")))
        )
}
