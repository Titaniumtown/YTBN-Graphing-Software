use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets");

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
		"static COMPLETION_HASHMAP: phf::Map<&'static str, &'static str> = {}",
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

const QUOTE: char = '"';
fn compile_hashmap() -> phf_codegen::Map<String> {
	let mut tuple_list: Vec<(String, String)> = Vec::new();

	for entry in SUPPORTED_FUNCTIONS
		.iter()
		.map(|entry| format!("{}(", entry))
		.collect::<Vec<String>>()
	{
		for i in 1..entry.len() {
			let (first, last) = entry.split_at(i);
			tuple_list.push((first.to_string(), last.to_string()));
		}
	}

	let mut output: phf_codegen::Map<String> = phf_codegen::Map::new();

	let key_list: Vec<String> = tuple_list.iter().map(|(key, _)| key.clone()).collect();

	let mut seen = HashSet::new();
	for (key, value) in tuple_list.clone() {
		if seen.contains(&key) {
			continue;
		}

		seen.insert(key.clone());

		let duplicate_num = key_list.iter().filter(|ele| **ele == key).count();
		if 1 == duplicate_num {
			output.entry(key, &(QUOTE.to_string() + &value + &QUOTE.to_string()));
			continue;
		}

		let same_keys_merged: Vec<String> = tuple_list
			.iter()
			.filter(|(a, _)| **a == key)
			.map(|(a, b)| a.clone() + b)
			.collect();

		let merged_key_value = key.clone() + &value;

		let mut common_substr: Option<String> = None;
		for same_key in same_keys_merged {
			if let Some(common_substr_unwrapped) = common_substr {
				common_substr = common_substring(&common_substr_unwrapped, &same_key);
			} else {
				common_substr = common_substring(&same_key, &merged_key_value)
			}

			if common_substr.is_none() {
				break;
			}
		}

		if let Some(common_substr_unwrapped) = common_substr {
			if !common_substr_unwrapped.is_empty() {
				output.entry(
					key.clone(),
					&(QUOTE.to_string()
						+ &common_substr_unwrapped.replace(&key, "")
						+ &QUOTE.to_string()),
				);
			}
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
