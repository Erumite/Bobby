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

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Bobby - No Nonsense Audio Player")
            .with_inner_size(Vec2::new(680.0, 480.0))
            .with_min_inner_size(Vec2::new(450.0, 300.0))
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Bobby Audio Player",
        native_options,
        Box::new(|cc| Box::new(BobbyApp::new(cc))),
    )
}

