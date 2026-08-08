mod audio;
mod config;
mod playlist;
mod ui;

use eframe::egui::Vec2;
use ui::BobbyApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Bobby - No Nonsense Audio Player")
            .with_inner_size(Vec2::new(680.0, 480.0))
            .with_min_inner_size(Vec2::new(450.0, 300.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Bobby Audio Player",
        native_options,
        Box::new(|cc| Box::new(BobbyApp::new(cc))),
    )
}
