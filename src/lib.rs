#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_convert)]
#![feature(const_default_impls)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_assume)]
#![feature(const_option_ext)]
#![feature(const_slice_index)]
#![feature(slice_split_at_unchecked)]
#![feature(inline_const)]

#[macro_use]
extern crate static_assertions;

mod consts;
mod data;
mod function_entry;
mod function_manager;
mod math_app;
mod misc;
mod style;
mod unicode_helper;
mod widgets;

pub use crate::{
	function_entry::{FunctionEntry, Riemann},
	math_app::AppSettings,
	misc::{
		// decimal_round,
		hashed_storage_create,
		hashed_storage_read,
		newtons_method,
		option_vec_printer,
		step_helper,
		EguiHelper,
	},
	unicode_helper::{to_chars_array, to_unicode_hash},
};

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;

		#[global_allocator]
		static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

		#[wasm_bindgen(start)]
		pub fn start() -> Result<(), wasm_bindgen::JsValue> {
			tracing::info!("Starting...");

			// Used in order to hook into `panic!()` to log in the browser's console
			tracing_wasm::set_as_global_default();

			eframe::start_web("canvas", eframe::WebOptions {
				follow_system_theme: false,
				default_theme: eframe::Theme::Dark
			},
				Box::new(|cc| Box::new(math_app::MathApp::new(cc))))
		}
	}
}
