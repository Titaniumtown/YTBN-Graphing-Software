use std::{intrinsics::assume, mem};

use crate::parsing::is_variable;

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
			.replace("pi", "π") // replace "pi" text with pi symbol
			.replace("**", "^") // support alternate manner of expressing exponents
			.replace("exp", "\u{1fc93}") //stop-gap solution to fix the `exp` function
			.chars()
			.collect::<Vec<char>>(),
	)
	.iter()
	.map(|x| x.replace("\u{1fc93}", "exp")) // Convert back to `exp` text
	.collect::<Vec<String>>()
}

// __REVIEW__
pub fn split_function_chars(chars: &[char]) -> Vec<String> {
	if chars.is_empty() {
		return Vec::new();
	}

	// No point in processing everything if there's only 1 character
	if chars.len() == 1 {
		return vec![chars[0].to_string()];
	}

	unsafe {
		assume(chars.len() > 1);
		assume(!chars.is_empty());
	}

	// Resulting split-up data
	let mut data: Vec<Vec<&char>> = Vec::with_capacity(chars.len());

	// Buffer used to store data ready to be appended
	let mut buffer: Vec<&char> = Vec::with_capacity(chars.len());

	struct BoolSlice {
		closing_parens: bool,
		number: bool,
		letter: bool,
		variable: bool,
		masked_num: bool,
		masked_var: bool,
	}

	impl BoolSlice {
		fn from_char(c: &char, prev_masked_num: bool, prev_masked_var: bool) -> Self {
			let isnumber = c.is_ascii_digit();
			let isvariable = is_variable(c);
			Self {
				closing_parens: c == &')',
				number: isnumber,
				letter: c.is_ascii_alphabetic(),
				variable: isvariable,
				masked_num: match isnumber {
					true => prev_masked_num,
					false => false,
				},
				masked_var: match isvariable {
					true => prev_masked_var,
					false => false,
				},
			}
		}
		fn is_variable(&self) -> bool { self.variable && !self.masked_var }

		fn is_number(&self) -> bool { self.number && !self.masked_num }
	}

	// Setup first char here
	let mut prev_char: BoolSlice = BoolSlice::from_char(&chars[0], false, false);
	buffer.push(&chars[0]);

	// Iterate through all chars excluding the first one
	for c in chars.iter().skip(1) {
		// Set data about current character
		let mut curr_c = BoolSlice::from_char(c, prev_char.masked_num, prev_char.masked_var);

		// If previous char was a masked number, and current char is a number, mask current char's variable status
		if prev_char.masked_num && curr_c.number {
			curr_c.masked_num = true;
		}

		// If previous char was a masked variable, and current char is a variable, mask current char's variable status
		if prev_char.masked_var && curr_c.variable {
			curr_c.masked_var = true;
		}

		// If letter and not a variable (or a masked variable)
		if prev_char.letter && !prev_char.is_variable() {
			// Mask number status if current char is number
			if curr_c.number {
				curr_c.masked_num = true;
			}

			// Mask variable status if current char is a variable
			if curr_c.variable {
				curr_c.masked_var = true;
			}
		}

		let mut do_split = false;

		if prev_char.closing_parens {
			// Cases like `)x`, `)2`, and `)(`
			do_split = (c == &'(')
				| (curr_c.letter && !curr_c.is_variable())
				| curr_c.is_variable()
				| curr_c.is_number();
		} else if c == &'(' {
			// Cases like `x(` and `2(`
			do_split = (prev_char.is_variable() | prev_char.is_number()) && !prev_char.letter;
		} else if prev_char.is_number() {
			// Cases like `2x` and `2sin(x)`
			do_split = curr_c.is_variable() | curr_c.letter;
		} else if curr_c.is_variable() | curr_c.letter {
			// Cases like `e2` and `xx`
			do_split = prev_char.is_number()
				| (prev_char.is_variable() && curr_c.is_variable())
				| prev_char.is_variable()
		} else if (curr_c.is_number() | curr_c.letter | curr_c.is_variable())
			&& (prev_char.is_number() | prev_char.letter)
		{
			do_split = true;
		} else if curr_c.is_number() && prev_char.is_variable() {
			// Cases like `x2`
			do_split = true;
		}

		// Append split
		if do_split {
			data.push(Vec::new());

			// Don't deinitialize `buffer`, simply swap data between the new last element of `data` with buffer!
			unsafe {
				mem::swap(data.last_mut().unwrap_unchecked(), &mut buffer);
			}
		}

		// Add character to buffer
		buffer.push(c);

		// Move current character data to `prev_char`
		prev_char = curr_c;
	}

	// If there is still data in the buffer, append it
	if !buffer.is_empty() {
		data.push(buffer);
	}

	data.into_iter()
		.map(|e| e.iter().map(|c| *c).collect::<String>())
		.collect::<Vec<String>>()
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
	pub fn is_none(&self) -> bool { matches!(&self, &Hint::None) }

	#[allow(dead_code)]
	pub fn is_some(&self) -> bool { !self.is_none() }

	#[allow(dead_code)]
	pub fn is_single(&self) -> bool { matches!(&self, &Hint::Single(_)) }
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
			("x", vec!["x"]),
			("xxx", vec!["x", "x", "x"]),
			("sin(cos(x)x)", vec!["sin(cos(x)", "x)"]),
		]);

		for (key, value) in values {
			assert_eq!(super::split_function(key), value);
		}
	}

	#[test]
	fn hint_tests() {
		{
			let hint = Hint::None;
			assert!(hint.is_none());
			assert!(!hint.is_some());
			assert!(!hint.is_single());
		}

		{
			let hint = Hint::Single(&"");
			assert!(!hint.is_none());
			assert!(hint.is_some());
			assert!(hint.is_single());
		}

		{
			let hint = Hint::Many(&[""]);
			assert!(!hint.is_none());
			assert!(hint.is_some());
			assert!(!hint.is_single());
		}
	}
}
