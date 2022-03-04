#![allow(clippy::unused_unit)] // Fixes clippy keep complaining about wasm_bindgen
#![feature(const_mut_refs)]

mod egui_app;
mod function;
mod misc;
mod parsing;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use misc::log_helper;
        use wasm_bindgen::prelude::*;

        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

        #[wasm_bindgen]
        pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
            log_helper("Initializing...");

            // Used in order to hook into `panic!()` to log in the browser's console
            log_helper("Initializing panic hooks...");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            log_helper("Initialized panic hooks!");

            log_helper("Finished initializing!");

            log_helper("Starting App...");
            let app = egui_app::MathApp::default();
            eframe::start_web(canvas_id, Box::new(app))
        }
    }
}
