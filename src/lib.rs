#![allow(clippy::unused_unit)] // Fixes clippy keep complaining about wasm_bindgen

mod egui_app;
mod function;
mod misc;

#[cfg(target_arch = "wasm32")]
use misc::log_helper;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[cfg(debug_assertions)]
fn init_tracing_wasm() {
    log_helper("Initializing tracing_wasm...");
    tracing_wasm::set_as_global_default();
    log_helper("Initialized tracing_wasm!");
}

#[cfg(target_arch = "wasm32")]
#[cfg(not(debug_assertions))]
fn init_tracing_wasm() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    log_helper("Initializing...");

    // See performance in browser profiler!
    init_tracing_wasm();

    // Used in order to hook into `panic!()` to log in the browser's console
    log_helper("Initializing panic hooks...");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log_helper("Initialized panic hooks!");

    log_helper("Finished initializing!");

    log_helper("Starting App...");
    let app = egui_app::MathApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
