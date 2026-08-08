mod audio;
mod config;
mod playlist;
mod ui;

use eframe::egui::Vec2;
use ui::BobbyApp;

fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_bytes = include_bytes!("../assets/bobby.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to load application icon")
            .to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() -> eframe::Result<()> {
    let icon = load_icon();
    let config = config::AppConfig::load();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Bobby")
        .with_min_inner_size(Vec2::new(450.0, 300.0))
        .with_icon(icon);

    if let (Some(w), Some(h)) = (config.window_width, config.window_height) {
        viewport = viewport.with_inner_size(Vec2::new(w, h));
    } else {
        viewport = viewport.with_inner_size(Vec2::new(680.0, 480.0));
    }

    if let (Some(x), Some(y)) = (config.window_x, config.window_y) {
        viewport = viewport.with_position(eframe::egui::Pos2::new(x, y));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Bobby",
        native_options,
        Box::new(|cc| Box::new(BobbyApp::new(cc))),
    )
}

