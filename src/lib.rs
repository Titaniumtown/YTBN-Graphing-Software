#![allow(clippy::unused_unit)] // Fixes clippy keep complaining about wasm_bindgen
#![allow(clippy::type_complexity)] // Clippy, my types are fine.

mod chart_manager;
mod egui_app;
mod misc;
mod function;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    log("Initializing...");

    // See performance in browser profiler!
    log("Initializing tracing_wasm...");
    tracing_wasm::set_as_global_default();
    log("Initialized tracing_wasm!");

    // Used in order to hook into `panic!()` to log in the browser's console
    log("Initializing console_error_panic_hook...");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log("Initialized console_error_panic_hook!");

    log("Finished initializing!");

    log("Starting App...");
    let app = egui_app::MathApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
