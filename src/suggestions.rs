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
			return output;
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
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

impl ToString for HintEnum<'static> {
	fn to_string(&self) -> String {
		match self {
			HintEnum::Single(single_data) => single_data.to_string(),
			HintEnum::Many(multi_data) => {
				let max_i: i32 = (multi_data.len() as i32) - 1;

				"[".to_owned()
					+ &multi_data
						.iter()
						.enumerate()
						.map(|(i, x)| {
							let mut tmp = r#"""#.to_string() + x + r#"""#;
							// Add comma and space if needed
							if max_i > i as i32 {
								tmp += ", ";
							}
							tmp
						})
						.collect::<Vec<String>>()
						.concat() + "]"
			}
			HintEnum::None => String::new(),
		}
	}
}

impl HintEnum<'static> {
	pub fn is_some(&self) -> bool { !matches!(self, HintEnum::None) }

	pub fn is_none(&self) -> bool { !self.is_some() }
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

/// Gets completion from `COMPLETION_HASHMAP`
pub fn get_completion(key: &str) -> Option<HintEnum<'static>> {
	// If key is empty, just return None
	if key.is_empty() {
		return None;
	}

	// Get and clone the recieved data
	COMPLETION_HASHMAP.get(key).cloned()
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
}
