mod egui_app;

// These 2 are needed for rust-analyzer to work in vscode.
mod chart_manager;
mod misc;
mod function;

// For running the program natively! (Because why not?)
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = egui_app::MathApp::default();
    let options = eframe::NativeOptions {
        transparent: true,
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
