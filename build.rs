fn main() {
    let _ = command_run::Command::with_args("./pack_assets.sh", &[""])
        .enable_capture()
        .run();

    println!("cargo:rerun-if-changed=.git/logs/HEAD"); // genius
    println!("cargo:rerun-if-changed=assets"); // genius
    shadow_rs::new().unwrap();
}
