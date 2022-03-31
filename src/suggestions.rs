use crate::misc::chars_take;

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint(input: String) -> HintEnum<'static> {
	if input.is_empty() {
		return HintEnum::Single("x^2");
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
		return HintEnum::Single(")");
	}

	let len = chars.len();

	for i in (1..=MAX_FUNC_LEN).rev().filter(|i| len >= *i) {
		if let Some(output) = get_completion(chars_take(&chars, i)) {
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
				multi_data.iter().map(|a| a.to_string()).collect::<String>()
			}
			HintEnum::None => String::new(),
		}
	}
}

impl HintEnum<'static> {
	pub fn get_single(&self) -> Option<String> {
		match self {
			HintEnum::Single(x) => Some(x.to_string()),
			_ => None,
		}
	}

	pub fn is_multi(&self) -> bool { matches!(self, HintEnum::Many(_)) }

	pub fn ensure_many(&self) -> &[&str] {
		match self {
			HintEnum::Many(data) => data,
			_ => panic!("ensure_many called on non-Many value"),
		}
	}
	pub fn is_some(&self) -> bool { !matches!(self, HintEnum::None) }

	pub fn is_none(&self) -> bool { !self.is_some() }

	#[allow(dead_code)]
	pub fn ensure_single(&self) -> &&str {
		match self {
			HintEnum::Single(data) => data,
			_ => panic!("ensure_single called on non-Single value"),
		}
	}

	#[allow(dead_code)]
	pub fn is_single(&self) -> bool { !self.is_multi() }
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

/// Gets completion from `COMPLETION_HASHMAP`
pub fn get_completion(key: String) -> Option<HintEnum<'static>> {
	if key.is_empty() {
		return None;
	}

	COMPLETION_HASHMAP.get(&key).cloned()
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
			("si", HintEnum::Many(&["gnum(", "n(", "nh("])),
			("log", HintEnum::Many(&["2(", "10("])),
			("cos", HintEnum::Many(&["(", "h("])),
		]);

		for (key, value) in values {
			println!("{} + {:?}", key, value);
			assert_eq!(generate_hint(key.to_string()), value);
		}
	}
}
