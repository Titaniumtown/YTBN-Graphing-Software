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

			use eframe::WebRunner;
			use tracing::metadata::LevelFilter;
			#[derive(Clone)]
			 #[wasm_bindgen]
			 pub struct WebHandle {
				 runner: WebRunner,
			 }

			 #[wasm_bindgen]
			 impl WebHandle {
				 /// Installs a panic hook, then returns.
				 #[allow(clippy::new_without_default)]
				 #[wasm_bindgen(constructor)]
				 pub fn new() -> Self {
					 // eframe::WebLogger::init(LevelFilter::Debug).ok();
					 tracing_wasm::set_as_global_default();

					 Self {
						 runner: WebRunner::new(),
					 }
				 }

				 /// Call this once from JavaScript to start your app.
				 #[wasm_bindgen]
				 pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
					 self.runner
						 .start(
							 canvas_id,
							 eframe::WebOptions::default(),
							 Box::new(|cc| Box::new(math_app::MathApp::new(cc))),
						 )
						 .await
				 }
			}

		#[wasm_bindgen(start)]
		pub async fn start() {
			tracing::info!("Starting...");


			let web_handle = WebHandle::new();
			web_handle.start("canvas").await.unwrap()
		}
	}
}
