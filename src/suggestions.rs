use crate::misc::chars_take;

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint(input: &str) -> HintEnum<'static> {
	if input.is_empty() {
		return HintEnum::Single("x^2");
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
		return HintEnum::Single(")");
	}

	let len = chars.len();

	for i in (1..=MAX_COMPLETION_LEN).rev().filter(|i| len >= *i) {
		if let Some(output) = get_completion(&chars_take(&chars, i)) {
			return output.clone();
		}
	}

	HintEnum::None
}

#[derive(Clone, PartialEq)]
pub enum HintEnum<'a> {
	Single(&'static str),
	Many(&'a [&'static str]),
	None,
}

impl std::fmt::Debug for HintEnum<'static> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self) }
}

impl std::fmt::Display for HintEnum<'static> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HintEnum::Single(single_data) => {
				return write!(f, "{}", single_data);
			}
			HintEnum::Many(multi_data) => {
				return write!(f, "{:?}", multi_data);
			}
			HintEnum::None => {
				return write!(f, "None");
			}
		}
	}
}

impl HintEnum<'static> {
	pub fn is_some(&self) -> bool { !matches!(self, HintEnum::None) }

	pub fn is_none(&self) -> bool { !self.is_some() }
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

/// Gets completion from `COMPLETION_HASHMAP`
pub fn get_completion(key: &str) -> Option<&HintEnum<'static>> {
	// If key is empty, just return None
	if key.is_empty() {
		return None;
	}

	// Get and clone the recieved data
	COMPLETION_HASHMAP.get(key)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;

	/// Tests to make sure hints are properly outputed based on input
	#[test]
	fn hint_test() {
		let values = HashMap::from([
			("", HintEnum::Single("x^2")),
			("si", HintEnum::Many(&["n(", "nh(", "gnum("])),
			("log", HintEnum::Many(&["2(", "10("])),
			("cos", HintEnum::Many(&["(", "h("])),
			("sin(", HintEnum::Single(")")),
			("sqrt", HintEnum::Single("(")),
		]);

		for (key, value) in values {
			println!("{} + {:?}", key, value);
			assert_eq!(generate_hint(key), value);
		}
	}

	#[test]
	fn hint_to_string_test() {
		let values = HashMap::from([
			("x^2", HintEnum::Single("x^2")),
			(
				r#"["n(", "nh(", "gnum("]"#,
				HintEnum::Many(&["n(", "nh(", "gnum("]),
			),
			(r#"["n("]"#, HintEnum::Many(&["n("])),
			("None", HintEnum::None),
		]);

		for (key, value) in values {
			assert_eq!(value.to_string(), key);
		}
	}
}
