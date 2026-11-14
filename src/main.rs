// Release builds are GUI applications: no console window when started from
// Explorer or another launcher. Debug builds keep the console for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod dialogs;
mod engine;
mod hooks;
mod hotkeys;
mod i18n;
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
            .with_inner_size([app::WINDOW_WIDTH, app::WINDOW_HEIGHT])
            .with_min_inner_size([app::WINDOW_WIDTH, app::WINDOW_HEIGHT])
            .with_max_inner_size([app::WINDOW_WIDTH, app::WINDOW_HEIGHT])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_maximize_button(false)
            .with_minimize_button(false)
            .with_app_id("SimpleAutoClicker"),
        centered: true,
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        app::BASE_TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, settings)))),
    )
}
