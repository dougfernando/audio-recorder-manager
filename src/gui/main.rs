// GUI entry point for Audio Recorder Manager using GPUI
// This binary provides a graphical user interface for the audio recorder

use gpui::*;
use rust_embed::RustEmbed;
use std::borrow::Cow;

pub mod app;
mod state;
mod services;
mod components;

use app::AudioRecorderApp;

/// An asset source that loads assets from the `./assets` folder.
#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow::anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

fn main() {
    env_logger::init();

    // Register Assets to GPUI application for icon support
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        // Initialize gpui-component system
        gpui_component::init(cx);

        // Load and apply Catppuccin Latte theme
        let theme_name = "Catppuccin Latte";
        if let Err(err) = gpui_component::ThemeRegistry::watch_dir(
            std::path::PathBuf::from("./themes"),
            cx,
            move |cx| {
                if let Some(theme) = gpui_component::ThemeRegistry::global(cx)
                    .themes()
                    .get(theme_name)
                    .cloned()
                {
                    gpui_component::Theme::global_mut(cx).apply_config(&theme);
                }
            },
        ) {
            log::error!("Failed to watch themes directory: {}", err);
        }

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
