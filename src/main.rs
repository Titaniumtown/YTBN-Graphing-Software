#![feature(const_mut_refs)]
#![feature(let_chains)]

#[macro_use]
extern crate static_assertions;

mod autocomplete_helper;
mod consts;
mod function_entry;
mod function_handling;
mod math_app;
mod misc;
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
