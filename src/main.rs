use eframe::{epi, egui};
mod egui_app;
mod misc;
mod chart_manager;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = egui_app::MathApp::default();
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: true,
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}