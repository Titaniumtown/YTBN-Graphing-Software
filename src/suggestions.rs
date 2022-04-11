use crate::misc::chars_take;

pub const HINTENUM_EMPTY: HintEnum = HintEnum::Single("x^2");
const HINTENUM_CLOSED_PARENS: HintEnum = HintEnum::Single(")");

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint<'a>(input: &str) -> &'a HintEnum<'a> {
	if input.is_empty() {
		return &HINTENUM_EMPTY;
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
		return &HINTENUM_CLOSED_PARENS;
	}

	let len = chars.len();

	for key in (1..=MAX_COMPLETION_LEN)
		.rev()
		.filter(|i| len >= *i)
		.map(|i| chars_take(&chars, i))
		.filter(|cut_string| !cut_string.is_empty())
	{
		if let Some(output) = COMPLETION_HASHMAP.get(&key) {
			return output;
		}
	}

	&HintEnum::None
}

#[derive(PartialEq)]
pub enum HintEnum<'a> {
	Single(&'a str),
	Many(&'a [&'a str]),
	None,
}

impl<'a> std::fmt::Debug for HintEnum<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self) }
}

impl<'a> std::fmt::Display for HintEnum<'a> {
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

impl<'a> HintEnum<'a> {
	pub fn is_none(&self) -> bool { matches!(self, HintEnum::None) }

	#[allow(dead_code)]
	pub fn is_some(&self) -> bool { !self.is_none() }

	pub fn is_single(&self) -> bool {
		match self {
			HintEnum::Single(_) => true,
			_ => false,
		}
	}
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

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
			assert_eq!(generate_hint(key), &value);
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
