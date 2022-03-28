#![allow(clippy::unused_unit)] // Fixes clippy keep complaining about wasm_bindgen
#![feature(const_mut_refs)]

#[macro_use]
extern crate static_assertions;

mod consts;
mod egui_app;
mod function;
mod function_output;
mod misc;
mod parsing;

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;

		#[global_allocator]
		static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

		#[wasm_bindgen(start)]
		pub fn start() -> Result<(), wasm_bindgen::JsValue> {
			tracing::info!("Initializing...");

			// Used in order to hook into `panic!()` to log in the browser's console
			tracing::info!("Initializing panic hooks...");
			console_error_panic_hook::set_once();
			tracing_wasm::set_as_global_default();
			tracing::info!("Initialized panic hooks!");

			tracing::info!("Finished initializing!");

			tracing::info!("Starting App...");
			eframe::start_web("canvas", Box::new(|cc| Box::new(egui_app::MathApp::new(cc))))
		}
	}
}
