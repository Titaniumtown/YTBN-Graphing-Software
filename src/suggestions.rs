use crate::misc::{chars_take, common_substring};
use std::collections::{HashMap, HashSet};

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint(input: &str) -> Option<String> {
	if input.is_empty() {
		return Some("x^2".to_owned());
	}

	let chars: Vec<char> = input.chars().collect();

	let mut open_parens: usize = 0;
	let mut closed_parens: usize = 0;
	chars.iter().for_each(|chr| match *chr {
		'(' => open_parens += 1,
		')' => closed_parens += 1,
		_ => {}
	});

	if open_parens > closed_parens {
		return Some(")".to_owned());
	}

	let len = chars.len();

	if len >= 5 {
		let result_five = get_completion(chars_take(&chars, 5));
		if result_five.is_some() {
			return result_five;
		}
	}

	if len >= 4 {
		let result_four = get_completion(chars_take(&chars, 4));
		if result_four.is_some() {
			return result_four;
		}
	}

	if len >= 3 {
		let result_three = get_completion(chars_take(&chars, 3));
		if result_three.is_some() {
			return result_three;
		}
	}

	if len >= 2 {
		let result_two = get_completion(chars_take(&chars, 2));
		if result_two.is_some() {
			return result_two;
		}
	}

	None
}

/// Creates hashmap used for function name completion
fn gen_completion_hashmap(input: Vec<String>) -> HashMap<String, String> {
	let mut tuple_list: Vec<(String, String)> = Vec::new();

	for entry in input {
		for i in 1..entry.len() {
			let (first, last) = entry.split_at(i);
			tuple_list.push((first.to_string(), last.to_string()));
		}
	}

	let mut output: HashMap<String, String> = HashMap::new();

	let key_list: Vec<String> = tuple_list.iter().map(|(key, _)| key.clone()).collect();

	let mut seen = HashSet::new();
	for (key, value) in tuple_list.clone() {
		if seen.contains(&key) {
			continue;
		}

		seen.insert(key.clone());

		let duplicate_num = key_list.iter().filter(|ele| **ele == key).count();
		if 1 == duplicate_num {
			output.insert(key, value);
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
				output.insert(key.clone(), common_substr_unwrapped.replace(&key, ""));
			}
		}
	}

	output
}

/// List of supported functions from exmex
const SUPPORTED_FUNCTIONS: [&str; 22] = [
	"abs", "signum", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "floor",
	"round", "ceil", "trunc", "fract", "exp", "sqrt", "cbrt", "ln", "log2", "log10",
];

lazy_static::lazy_static! {
	static ref COMPLETION_HASHMAP: HashMap<String, String> = gen_completion_hashmap(SUPPORTED_FUNCTIONS.to_vec().iter().map(|ele| ele.to_string() + "(").collect());
}

/// Gets completion from `COMPLETION_HASHMAP`
pub fn get_completion(key: String) -> Option<String> {
	if key.is_empty() {
		return None;
	}

	match COMPLETION_HASHMAP.get(&key) {
		Some(data_x) => {
			if data_x.is_empty() {
				None
			} else {
				Some(data_x.clone())
			}
		}
		None => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests to make sure hints are properly outputed based on input
	#[test]
	fn hint_test() {
		let values = HashMap::from([
			("", "x^2"),
			("sin(x", ")"),
			("sin(x)", ""),
			("x^x", ""),
			("(x+1)(x-1", ")"),
			("lo", "g"),
			("log", ""), // because there are multiple log functions
			("asi", "n("),
			("asin", "("),
			("fl", "oor("),
			("ata", "n("),
			("at", "an("),
			("roun", "d("),
			("floo", "r("),
			("flo", "or("),
		]);

		for (key, value) in values {
			println!("{} + {}", key, value);
			assert_eq!(generate_hint(key).unwrap_or_default(), value.to_owned());
		}
	}

	#[test]
	fn completion_hashmap_test() {
		let values = hashmap_test_gen();
		for (key, value) in values {
			println!(
				"{} + {}",
				key,
				match value.clone() {
					Some(x) => x.clone(),
					None => "(No completion)".to_string(),
				}
			);

			assert_eq!(get_completion(key.to_string()), value);
		}
	}

	fn hashmap_test_gen() -> HashMap<String, Option<String>> {
		let mut values: HashMap<String, Option<String>> = HashMap::new();

		let processed_func: Vec<String> = SUPPORTED_FUNCTIONS
			.iter()
			.map(|ele| ele.to_string() + "(")
			.collect();

		let mut data_tuple: Vec<(String, Option<String>)> = Vec::new();
		for func in processed_func.iter() {
			for i in 1..=func.len() {
				let (first, last) = func.split_at(i);
				let value = match last {
					"" => None,
					x => Some(x.to_string()),
				};
				data_tuple.push((first.to_string(), value));
			}
		}

		let key_list: Vec<String> = data_tuple.iter().map(|(a, _)| a.clone()).collect();

		for (key, value) in data_tuple {
			if key_list.iter().filter(|a| **a == key).count() == 1 {
				values.insert(key, value);
			}
		}

		let values_old = values.clone();
		values = values
			.iter()
			.filter(|(key, _)| values_old.iter().filter(|(a, _)| a == key).count() == 1)
			.map(|(a, b)| (a.to_string(), b.clone()))
			.collect();

		let manual_values: Vec<(&str, Option<&str>)> =
			vec![("sin", None), ("cos", None), ("tan", None)];

		for (key, value) in manual_values {
			values.insert(key.to_string(), value.map(|x| x.to_string()));
		}
		values
	}
}
