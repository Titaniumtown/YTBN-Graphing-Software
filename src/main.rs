#![feature(const_mut_refs)]
#![feature(let_chains)]
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

// For running the program natively! (Because why not?)
#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let subscriber = tracing_subscriber::FmtSubscriber::builder()
		.with_max_level(tracing::Level::TRACE)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	eframe::run_native(
		"(Yet-to-be-named) Graphing Software",
		eframe::NativeOptions::default(),
		Box::new(|cc| Box::new(math_app::MathApp::new(cc))),
	);
}
