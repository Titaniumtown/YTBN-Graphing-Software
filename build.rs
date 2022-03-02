fn main() {
    println!("cargo:rerun-if-changed=.git/refs/heads/main"); // genius
    shadow_rs::new().unwrap();
}
