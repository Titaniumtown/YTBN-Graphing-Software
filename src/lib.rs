#![feature(const_mut_refs)]
#![feature(let_chains)]
#![feature(stmt_expr_attributes)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_convert)]
#![feature(const_default_impls)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_assume)]
#![feature(const_option_ext)]
#![feature(const_slice_index)]

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate uuid;

mod consts;
mod data;
mod function_entry;
mod function_manager;
mod math_app;
mod misc;
mod widgets;

pub use crate::{
	function_entry::{FunctionEntry, Riemann},
	math_app::AppSettings,
	misc::{
		decimal_round, format_bytes, hashed_storage_create, hashed_storage_read,
		option_vec_printer, resolution_helper, step_helper, SteppedVector,
	},
};

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;

		#[global_allocator]
		static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

		#[wasm_bindgen(start)]
		pub fn start() -> Result<(), wasm_bindgen::JsValue> {
			tracing::info!("Initializing...");
			// Used in order to hook into `panic!()` to log in the browser's console
			tracing_wasm::set_as_global_default();
			tracing::info!("Starting App...");
			eframe::start_web("canvas", Box::new(|cc| Box::new(math_app::MathApp::new(cc))))
		}
	}
}
