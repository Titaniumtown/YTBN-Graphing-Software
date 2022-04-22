fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets/*");

	let _ = command_run::Command::with_args("../pack_assets.sh", &[""])
		.enable_capture()
		.run();

	shadow_rs::new().expect("Could not initialize shadow_rs");
}
