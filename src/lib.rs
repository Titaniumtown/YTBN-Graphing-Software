#![feature(const_mut_refs)]
#![feature(let_chains)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_assume)]
#![feature(const_option_ext)]
#![feature(const_slice_index)]
#![feature(slice_split_at_unchecked)]
#![feature(inline_const)]
#![feature(const_for)]

#[macro_use]
extern crate static_assertions;

mod consts;
mod function_entry;
mod function_manager;
mod math_app;
mod misc;
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
		HashBytes,
	},
	unicode_helper::{to_chars_array, to_unicode_hash},
};

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;

		use lol_alloc::{FreeListAllocator, LockedAllocator};
		#[global_allocator]
				static ALLOCATOR: LockedAllocator<FreeListAllocator> = LockedAllocator::new(FreeListAllocator::new());

		#[wasm_bindgen(start)]
		pub async fn start() {
			tracing::info!("Starting...");

			// Used in order to hook into `panic!()` to log in the browser's console
			tracing_wasm::set_as_global_default();


			eframe::start_web("canvas", eframe::WebOptions::default(),
				Box::new(|cc| Box::new(math_app::MathApp::new(cc)))).await.unwrap();
		}
	}
}
