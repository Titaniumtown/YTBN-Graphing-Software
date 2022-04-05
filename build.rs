use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const SUPPORTED_FUNCTIONS: [&str; 22] = [
	"abs", "signum", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "floor",
	"round", "ceil", "trunc", "fract", "exp", "sqrt", "cbrt", "ln", "log2", "log10",
];

fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets/*");

	let _ = command_run::Command::with_args("./pack_assets.sh", &[""])
		.enable_capture()
		.run();
	shadow_rs::new().unwrap();

	generate_hashmap();
}

fn generate_hashmap() {
	let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
	let mut file = BufWriter::new(File::create(&path).unwrap());
	let max_len: usize = SUPPORTED_FUNCTIONS
		.to_vec()
		.iter()
		.map(|func| func.len())
		.max()
		.unwrap();

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
		"static COMPLETION_HASHMAP: phf::Map<&'static str, HintEnum> = {}",
		hashmap.build()
	)
	.unwrap();
	writeln!(&mut file, ";").unwrap();

	writeln!(
		&mut file,
		"const MAX_COMPLETION_LEN: usize = {};",
		max_len
	)
	.unwrap();

	write!(
		&mut file,
		"#[allow(dead_code)] pub const SUPPORTED_FUNCTIONS: [&str; {}] = {:?};",
		SUPPORTED_FUNCTIONS.len(),
		SUPPORTED_FUNCTIONS.to_vec()
	)
	.unwrap();
}

include!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/src/autocomplete_helper.rs"
));
