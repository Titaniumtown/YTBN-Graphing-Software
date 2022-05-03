use crate::parsing::{is_letter, is_number, is_variable};

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
	split_function_chars(
		&input
			.replace("pi", "π")
			.replace("**", "^")
			.replace("exp", "\u{1fc93}")
			.chars()
			.collect::<Vec<char>>(),
	)
	.iter()
	.map(|x| x.replace("\u{1fc93}", "exp"))
	.collect::<Vec<String>>()
}

fn split_function_chars(chars: &[char]) -> Vec<String> {
	assert!(!chars.is_empty());

	let mut split: Vec<String> = Vec::new();

	let mut buffer: Vec<char> = Vec::new();

	#[derive(Default)]
	struct BoolSlice {
		closing_parens: bool,
		number: bool,
		letter: bool,
		variable: bool,
		masked_num: bool,
		masked_var: bool,
		exists: bool,
	}

	impl BoolSlice {
		#[inline]
		fn is_variable(&self) -> bool { self.variable && !self.masked_var }

		#[inline]
		fn is_number(&self) -> bool { self.number && !self.masked_num }
	}
	let mut prev_char: BoolSlice = BoolSlice::default();

	for c in chars {
		let mut curr_c = BoolSlice {
			closing_parens: c == &')',
			number: is_number(c),
			letter: is_letter(c),
			variable: is_variable(c),
			masked_num: if is_number(c) {
				prev_char.masked_num
			} else {
				false
			},
			masked_var: if is_variable(c) {
				prev_char.masked_var
			} else {
				false
			},
			exists: true,
		};

		let buffer_string = buffer.iter().collect::<String>();

		// Check if prev_char is valid
		if prev_char.exists {
			// if previous char was a masked number, and current char is a number, mask current char's variable status
			if prev_char.masked_num && curr_c.number {
				curr_c.masked_num = true;
			}

			// if previous char was a masked variable, and current char is a variable, mask current char's variable status
			if prev_char.masked_var && curr_c.variable {
				curr_c.masked_var = true;
			}

			// if letter and not a variable (or a masked variable)
			if prev_char.letter && !(prev_char.variable && !prev_char.masked_var) {
				// mask number status if current char is number
				if curr_c.number {
					curr_c.masked_num = true;
				}

				// mask variable status if current char is a variable
				if curr_c.variable {
					curr_c.masked_var = true;
				}
			}
		}

		let mut do_split = false;

		if prev_char.closing_parens {
			// cases like `)x`, `)2`, and `)(`
			if (c == &'(')
				| (curr_c.letter && !curr_c.is_variable())
				| curr_c.is_variable()
				| curr_c.is_number()
			{
				do_split = true;
			}
		} else if c == &'(' {
			// cases like `x(` and `2(`
			if (prev_char.is_variable() | prev_char.is_number()) && !prev_char.letter {
				do_split = true;
			}
		} else if prev_char.is_number() {
			// cases like `2x` and `2sin(x)`
			if curr_c.is_variable() | curr_c.letter {
				do_split = true;
			}
		} else if curr_c.is_variable() | curr_c.letter {
			// cases like `e2` and `xx`
			if prev_char.is_number()
				| (prev_char.is_variable() && curr_c.is_variable())
				| prev_char.is_variable()
			{
				do_split = true;
			}
		} else if (curr_c.is_number() | curr_c.letter | curr_c.is_variable())
			&& (prev_char.is_number() | prev_char.letter)
		{
			// cases like `x2` and `xx`
			do_split = true;
		} else if curr_c.is_number() && prev_char.is_variable() {
			do_split = true;
		}

		// split and append buffer
		if do_split {
			split.push(buffer_string);
			buffer.clear();
		}

		buffer.push(*c);
		prev_char = curr_c;
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

impl<'a> std::fmt::Debug for Hint<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Display::fmt(self, f)
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
			assert_eq!(super::generate_hint(key), &value);
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
				let split = super::split_function(&key);

				if split.len() != 1 {
					panic!("failed: {} (len: {}, split: {:?})", key, split.len(), split);
				}

				let generated_hint = super::generate_hint(&key);
				if generated_hint.is_none() {
					println!("success: {}", key);
				} else {
					panic!("failed: {} (Hint: '{}')", key, generated_hint.to_string());
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
			("emax(x)", vec!["e", "max(x)"]),
		]);

		for (key, value) in values {
			assert_eq!(super::split_function(key), value);
		}
	}
}
