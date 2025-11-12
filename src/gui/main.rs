// GUI entry point for Audio Recorder Manager using GPUI
// This binary provides a graphical user interface for the audio recorder

use gpui::*;

pub mod app;
mod state;
mod services;
mod components;

use app::AudioRecorderApp;

fn main() {
    env_logger::init();

    let app = Application::new();

    app.run(move |cx| {
        // Initialize gpui-component system
        gpui_component::init(cx);

        // Configure app settings
        cx.activate(true);

        // Open main window
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::default(),
                        size: Size {
                            width: px(1024.0),
                            height: px(768.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Audio Recorder Manager".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    window_min_size: Some(Size {
                        width: px(800.0),
                        height: px(600.0),
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| AudioRecorderApp::new(window, cx));
                    cx.new(|cx| gpui_component::Root::new(view.into(), window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
