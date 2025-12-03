#[macro_use]
extern crate static_assertions;

mod consts;
mod function_entry;
mod function_manager;
mod math_app;
mod misc;
mod unicode_helper;
mod widgets;

// For running the program natively! (Because why not?)
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    eframe::run_native(
        "(Yet-to-be-named) Graphing Software",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(math_app::MathApp::new(cc)))),
    )
}
