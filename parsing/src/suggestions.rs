use crate::parsing::is_number;

pub const HINT_EMPTY: Hint = Hint::Single("x^2");
const HINT_CLOSED_PARENS: Hint = Hint::Single(")");

/// Only enacts println if cfg(test) is enabled
#[allow(unused_macros)]
macro_rules! test_print {
    ($($arg:tt)*) => {
		#[cfg(test)]
        println!($($arg)*)
    };
}

pub fn split_function(input: &str) -> Vec<String> {
	split_function_chars(&input.chars().collect::<Vec<char>>())
}

fn split_function_chars(chars: &[char]) -> Vec<String> {
	assert!(!chars.is_empty());

	let mut split: Vec<String> = Vec::new();

	let mut buffer: Vec<char> = Vec::new();
	for c in chars {
		buffer.push(*c);
		if *c == ')' {
			split.push(buffer.iter().collect::<String>());
			buffer.clear();
			continue;
		}

		let buffer_string = buffer.iter().collect::<String>();

		if ((&buffer_string == "log") | (&buffer_string == "log1")) && is_number(&c) {
			continue;
		}
	}

	if !buffer.is_empty() {
		split.push(buffer.iter().collect::<String>());
	}
	split
}

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint<'a>(input: &str) -> &'a Hint<'a> {
	if input.is_empty() {
		return &HINT_EMPTY;
	}

	let chars: Vec<char> = input.chars().collect::<Vec<char>>();

	let mut open_parens: usize = 0;
	let mut closed_parens: usize = 0;
	chars.iter().for_each(|chr| match *chr {
		'(' => open_parens += 1,
		')' => closed_parens += 1,
		_ => {}
	});

	if open_parens > closed_parens {
		return &HINT_CLOSED_PARENS;
	}

	COMPLETION_HASHMAP
		.get(&unsafe { split_function_chars(&chars).last().unwrap_unchecked() }.as_str())
		.unwrap_or(&Hint::None)
}

#[derive(PartialEq)]
pub enum Hint<'a> {
	Single(&'a str),
	Many(&'a [&'a str]),
	None,
}

impl<'a> std::fmt::Debug for Hint<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self) }
}

impl<'a> std::fmt::Display for Hint<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Hint::Single(single_data) => {
				return write!(f, "{}", single_data);
			}
			Hint::Many(multi_data) => {
				return write!(f, "{:?}", multi_data);
			}
			Hint::None => {
				return write!(f, "None");
			}
		}
	}
}

impl<'a> Hint<'a> {
	pub fn is_none(&self) -> bool { matches!(self, Hint::None) }

	#[allow(dead_code)]
	pub fn is_some(&self) -> bool { !self.is_none() }

	#[allow(dead_code)]
	pub fn is_single(&self) -> bool { matches!(self, Hint::Single(_)) }
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;

	/// Tests to make sure hints are properly outputed based on input
	#[test]
	fn hints() {
		let values = HashMap::from([
			("", Hint::Single("x^2")),
			("si", Hint::Many(&["n(", "nh(", "gnum("])),
			("log", Hint::Many(&["2(", "10("])),
			("cos", Hint::Many(&["(", "h("])),
			("sin(", Hint::Single(")")),
			("sqrt", Hint::Single("(")),
			("ln(x)", Hint::None),
			("ln(x)cos", Hint::Many(&["(", "h("])),
		]);

		for (key, value) in values {
			println!("{} + {:?}", key, value);
			assert_eq!(generate_hint(key), &value);
		}
	}

	#[test]
	fn hint_to_string() {
		let values = HashMap::from([
			("x^2", Hint::Single("x^2")),
			(
				r#"["n(", "nh(", "gnum("]"#,
				Hint::Many(&["n(", "nh(", "gnum("]),
			),
			(r#"["n("]"#, Hint::Many(&["n("])),
			("None", Hint::None),
		]);

		for (key, value) in values {
			assert_eq!(value.to_string(), key);
		}
	}

	#[test]
	fn invalid_function() {
		SUPPORTED_FUNCTIONS
			.iter()
			.map(|func1| {
				SUPPORTED_FUNCTIONS
					.iter()
					.map(|func2| func1.to_string() + func2)
					.collect::<Vec<String>>()
			})
			.flatten()
			.filter(|func| !SUPPORTED_FUNCTIONS.contains(&func.as_str()))
			.for_each(|key| {
				println!("{}", key);
				if generate_hint(&key).is_none() {
					println!("success: {}", key);
				} else {
					panic!("failed: {}", key);
				}
			});
	}

	#[test]
	fn split_function() {
		let values = HashMap::from([
			("cos(x)", vec!["cos(x)"]),
			("cos(", vec!["cos("]),
			("cos(x)sin(x)", vec!["cos(x)", "sin(x)"]),
			("aaaaaaaaaaa", vec!["aaaaaaaaaaa"]),
		]);

		for (key, value) in values {
			assert_eq!(super::split_function(key), value);
		}
	}
}
