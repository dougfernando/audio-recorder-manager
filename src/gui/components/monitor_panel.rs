// Monitor panel component for viewing recording progress

use gpui::*;
use gpui_component::{button::*, ActiveTheme, IconName, Sizable, StyledExt};
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
    let recording_state = props.recording_state;

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
                .child(if recording_state.is_some() { "Recording in Progress" } else { "No Active Recording" })
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
                        .child(recording_state.map(|rs| format!("Progress: {}%", rs.progress)).unwrap_or_else(|| "Progress: 0%".to_string()))
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child(
                    recording_state.map(|rs| {
                        use audio_recorder_manager::RecordingDuration;
                        // Get total duration in seconds
                        let total = match &rs.duration {
                            RecordingDuration::Fixed(secs) => *secs,
                            RecordingDuration::Manual { max } => *max,
                        };
                        format!("Time: {}s / {}s", rs.elapsed, total)
                    }).unwrap_or_else(|| "Time: 0s / 0s".to_string())
                ))
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(SPACING_SM))
                .p(px(SPACING_MD))
                .bg(if recording_state.map(|rs| rs.has_audio).unwrap_or(false) {
                    cx.theme().success.opacity(0.1)
                } else {
                    cx.theme().muted.opacity(0.1)
                })
                .border_1()
                .border_color(if recording_state.map(|rs| rs.has_audio).unwrap_or(false) {
                    cx.theme().success.opacity(0.3)
                } else {
                    cx.theme().muted.opacity(0.3)
                })
                .rounded_lg()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().font_semibold().text_color(cx.theme().foreground).child(
                            recording_state.map(|rs| rs.device.clone()).unwrap_or_else(|| "Device".to_string())
                        ))
                        .child(div().text_color(
                            if recording_state.map(|rs| rs.has_audio).unwrap_or(false) {
                                cx.theme().success
                            } else {
                                cx.theme().muted_foreground
                            }
                        ).child(if recording_state.map(|rs| rs.has_audio).unwrap_or(false) {
                            "Audio Detected"
                        } else {
                            "No Audio"
                        }))
                )
                .child(div().text_sm().text_color(cx.theme().foreground).child(
                    recording_state.map(|rs| {
                        format!("{} frames | Sample Rate: {} Hz | {} channel(s)",
                            rs.frames_captured,
                            rs.sample_rate,
                            rs.channels
                        )
                    }).unwrap_or_else(|| "0 frames | Sample Rate: 0 Hz | 0 channels".to_string())
                ))
        )
        .child(
            Button::new("stop_recording")
                .danger()
                .icon(IconName::CircleX)
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
