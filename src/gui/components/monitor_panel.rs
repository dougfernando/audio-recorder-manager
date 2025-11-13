// Monitor panel component for viewing recording progress

use gpui::*;
use gpui_component::{button::*, ActiveTheme, Icon, IconName};
use crate::app::AudioRecorderApp;
use crate::state::RecordingState;

use super::header::{SPACING_SM, SPACING_MD, SPACING_LG};

pub struct MonitorPanelProps<'a> {
    pub recording_state: Option<&'a RecordingState>,
}

pub fn render_monitor_panel(
    props: &MonitorPanelProps,
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
                .child("Recording in Progress")
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
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child("Session Information")
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child(
                    props.recording_state
                        .map(|rs| format!("Session ID: {}", rs.session_id))
                        .unwrap_or_else(|| "Session ID: (Not recording)".to_string())
                ))
                .child(div().text_sm().text_color(cx.theme().foreground).child(
                    props.recording_state
                        .map(|rs| format!("File: {}", rs.filename))
                        .unwrap_or_else(|| "File: (No active recording)".to_string())
                ))
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
                        .child("Progress: 67%")
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child("Time: 20s / 30s"))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(cx.theme().success.opacity(0.1))
                .border_1()
                .border_color(cx.theme().success.opacity(0.3))
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().font_semibold().text_color(cx.theme().foreground).child("System Audio (Loopback)"))
                        .child(div().text_color(cx.theme().success).child("Audio Detected"))
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child("1,440 frames | Sample Rate: 48,000 Hz"))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(cx.theme().success.opacity(0.1))
                .border_1()
                .border_color(cx.theme().success.opacity(0.3))
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().font_semibold().text_color(cx.theme().foreground).child("Microphone"))
                        .child(div().text_color(cx.theme().success).child("Audio Detected"))
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child("1,425 frames | Sample Rate: 48,000 Hz (matched)"))
        )
        .child(
            Button::new("stop_recording")
                .danger()
                .icon(IconName::Square)
                .label("STOP RECORDING")
                .large()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.stop_recording(window, cx);
                }))
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .italic()
                .child("The recording is happening in the background. You can minimize this window.")
        )
}
