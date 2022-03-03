fn main() {
    println!("cargo:rerun-if-changed=.git/logs/HEAD"); // genius
    shadow_rs::new().unwrap();
}
