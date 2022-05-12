use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// REMEMBER TO UPDATE THIS IF EXMEX ADDS NEW FUNCTIONS
const SUPPORTED_FUNCTIONS: [&str; 22] = [
	"abs", "signum", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "floor",
	"round", "ceil", "trunc", "fract", "exp", "sqrt", "cbrt", "ln", "log2", "log10",
];

fn main() {
	println!("cargo:rerun-if-changed=src/*");

	generate_hashmap();
}

fn generate_hashmap() {
	let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
	let mut file = BufWriter::new(File::create(&path).expect("Could not create file"));

	let string_hashmap = compile_hashmap(
		SUPPORTED_FUNCTIONS
			.to_vec()
			.iter()
			.map(|a| a.to_string())
			.collect(),
	);

	let mut hashmap = phf_codegen::Map::new();

	for (key, value) in string_hashmap.iter() {
		hashmap.entry(key, value);
	}

	write!(
		&mut file,
		"static COMPLETION_HASHMAP: phf::Map<&'static str, Hint> = {};",
		hashmap.build()
	)
	.expect("Could not write to file");

	write!(
		&mut file,
		"#[allow(dead_code)] pub const SUPPORTED_FUNCTIONS: [&str; {}] = {:?};",
		SUPPORTED_FUNCTIONS.len(),
		SUPPORTED_FUNCTIONS.to_vec()
	)
	.expect("Could not write to file");
}

include!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/src/autocomplete_hashmap.rs"
));
