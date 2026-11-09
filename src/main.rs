mod app;
mod dialogs;
mod engine;
mod hooks;
mod hotkeys;
mod keys;
mod native;
mod record;
mod settings;
mod theme;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    let settings = settings::Settings::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(app::BASE_TITLE)
            .with_inner_size([460.0, 510.0])
            .with_min_inner_size([380.0, 220.0])
            .with_resizable(false)
            .with_app_id("SimpleAutoClicker"),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        app::BASE_TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, settings)))),
    )
}
