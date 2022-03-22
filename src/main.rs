#![feature(const_mut_refs)]

mod egui_app;
mod function;
mod function_output;
mod misc;
mod parsing;

// For running the program natively! (Because why not?)
#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let subscriber = tracing_subscriber::FmtSubscriber::builder()
		.with_max_level(tracing::Level::TRACE)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let options = eframe::NativeOptions {
		transparent: true,
		drag_and_drop_support: true,
		..Default::default()
	};

	eframe::run_native(
		"(Yet-to-be-named) Graphing Software",
		options,
		Box::new(|cc| Box::new(egui_app::MathApp::new(cc))),
	);
}
