use itertools::Itertools;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

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

	write!(
		&mut file,
		"static COMPLETION_HASHMAP: phf::Map<&'static str, HintEnum> = {}",
		compile_hashmap().build()
	)
	.unwrap();
	write!(&mut file, ";\n").unwrap();
}

/// List of supported functions from exmex
const SUPPORTED_FUNCTIONS: [&str; 22] = [
	"abs", "signum", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "floor",
	"round", "ceil", "trunc", "fract", "exp", "sqrt", "cbrt", "ln", "log2", "log10",
];

const CONST: char = '"';

fn compile_hashmap() -> phf_codegen::Map<String> {
	let functions_processed: Vec<String> = SUPPORTED_FUNCTIONS
		.iter()
		.map(|e| e.to_string() + "(")
		.collect();

	let mut seen = HashSet::new();

	let powerset = functions_processed
		.into_iter()
		.map(|func| func.chars().collect::<Vec<char>>())
		.powerset()
		.flatten()
		.filter(|e| e.len() > 1)
		.filter(|ele| {
			if seen.contains(ele) {
				return false;
			} else {
				seen.insert(ele.clone());
				return true;
			}
		})
		.collect::<Vec<Vec<char>>>();

	let mut tuple_list_1: Vec<(String, String)> = Vec::new();

	let mut seen_2: HashSet<(String, String)> = HashSet::new();
	for ele in powerset {
		for i in 1..ele.len() {
			let string = ele.clone().into_iter().collect::<String>();
			let (first, last) = string.split_at(i);
			let data = (first.to_string(), last.to_string());
			if seen_2.contains(&data) {
				continue;
			}
			seen_2.insert(data.clone());
			tuple_list_1.push(data)
		}
	}

	let keys: Vec<&String> = tuple_list_1.iter().map(|(a, _)| a).collect();
	let mut output = phf_codegen::Map::new();
	let mut seen_3: HashSet<String> = HashSet::new();

	for (key, value) in tuple_list_1.iter() {
		if seen_3.contains(&*key) {
			continue;
		}

		seen_3.insert(key.clone());
		if keys.iter().filter(|a| a == &&key).count() == 1 {
			output.entry(
				key.clone(),
				&format!("HintEnum::Single({}{}{})", CONST, value, CONST),
			);
		} else {
			let multi_data = tuple_list_1
				.iter()
				.filter(|(a, _)| a == key)
				.map(|(_, b)| b)
				.collect::<Vec<&String>>();
			output.entry(key.clone(), &format!("HintEnum::Many(&{:?})", multi_data));
		}
	}

	output
}

fn common_substring<'a>(a: &'a str, b: &'a str) -> Option<String> {
	let a_chars: Vec<char> = a.chars().collect();
	let b_chars: Vec<char> = b.chars().collect();
	if a_chars[0] != b_chars[0] {
		return None;
	}

	let mut last_value: String = a_chars[0].to_string();
	let max_common_i = std::cmp::min(a.len(), b.len()) - 1;
	for i in 1..=max_common_i {
		let a_i = a_chars[i];
		let b_i = b_chars[i];
		if a_i == b_i {
			last_value += &a_i.to_string()
		} else {
			break;
		}
	}

	Some(last_value)
}
