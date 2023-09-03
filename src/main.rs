#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod win32;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    win32::raise_privilege();

    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = false;
    native_options.default_theme = eframe::Theme::Light;
    // native_options.initial_window_size = Some(Vec2::new(970.0, 790.0)); // deal with high dpi?

    eframe::run_native(
        "看雪法兰城",
        native_options,
        Box::new(|cc| Box::new(app::MyApp::new(cc))),
    )
}
