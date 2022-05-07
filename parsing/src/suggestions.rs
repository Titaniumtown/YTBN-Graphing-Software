use std::intrinsics::assume;

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
			.replace("exp", "\u{1fc93}") // stop-gap solution to fix the `exp` function
			.chars()
			.collect::<Vec<char>>(),
	)
	.iter()
	.map(|x| x.replace("\u{1fc93}", "exp")) // Convert back to `exp` text
	.collect::<Vec<String>>()
}

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
	let mut data: Vec<String> = Vec::with_capacity(chars.len());

	// Need to start out with an empty string
	data.push(String::new());

	/// Used to store info about a character
	struct BoolSlice {
		closing_parens: bool,
		number: bool,
		letter: bool,
		variable: bool,
		masked_num: bool,
		masked_var: bool,
	}

	impl BoolSlice {
		const fn from_char(c: &char, prev_masked_num: bool, prev_masked_var: bool) -> Self {
			let isnumber = c.is_ascii_digit();
			let isvariable = is_variable(c);
			Self {
				closing_parens: *c == ')',
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
		const fn is_variable(&self) -> bool { self.variable && !self.masked_var }

		const fn is_number(&self) -> bool { self.number && !self.masked_num }

		const fn splitable(&self, c: &char, other: &BoolSlice) -> bool {
			if other.closing_parens {
				// Cases like `)x`, `)2`, and `)(`
				return (*c == '(')
					| (self.letter && !self.is_variable())
					| self.is_variable() | self.is_number();
			} else if *c == '(' {
				// Cases like `x(` and `2(`
				return (other.is_variable() | other.is_number()) && !other.letter;
			} else if other.is_number() {
				// Cases like `2x` and `2sin(x)`
				return self.is_variable() | self.letter;
			} else if self.is_variable() | self.letter {
				// Cases like `e2` and `xx`
				return other.is_number()
					| (other.is_variable() && self.is_variable())
					| other.is_variable();
			} else if (self.is_number() | self.letter | self.is_variable())
				&& (other.is_number() | other.letter)
			{
				return true;
			} else if self.is_number() && other.is_variable() {
				// Cases like `x2`
				return true;
			} else {
				return false;
			}
		}
	}

	// Setup first char here
	let mut prev_char: BoolSlice = BoolSlice::from_char(&chars[0], false, false);

	let mut last = unsafe { data.last_mut().unwrap_unchecked() };
	last.push(chars[0]);

	// Iterate through all chars excluding the first one
	for c in chars.iter().skip(1) {
		// Set data about current character
		let mut curr_c = BoolSlice::from_char(c, prev_char.masked_num, prev_char.masked_var);

		if prev_char.masked_num && curr_c.number {
			// If previous char was a masked number, and current char is a number, mask current char's variable status
			curr_c.masked_num = true;
		} else if prev_char.masked_var && curr_c.variable {
			// If previous char was a masked variable, and current char is a variable, mask current char's variable status
			curr_c.masked_var = true;
		} else if prev_char.letter && !prev_char.is_variable() {
			// If letter and not a variable (or a masked variable)
			if curr_c.number {
				// Mask number status if current char is number
				curr_c.masked_num = true;
			} else if curr_c.variable {
				// Mask variable status if current char is a variable
				curr_c.masked_var = true;
			}
		}

		// Append split
		if curr_c.splitable(c, &prev_char) {
			data.push(String::new());
			last = unsafe { data.last_mut().unwrap_unchecked() };
		}

		last.push(*c);

		// Move current character data to `prev_char`
		prev_char = curr_c;
	}

	data
}

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint<'a>(input: &str) -> &'a Hint<'a> {
	if input.is_empty() {
		return &HINT_EMPTY;
	}

	let chars: Vec<char> = input.chars().collect::<Vec<char>>();

	unsafe {
		assume(!chars.is_empty());
	}

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
	pub const fn is_none(&self) -> bool { matches!(&self, &Hint::None) }

	#[allow(dead_code)]
	pub const fn is_some(&self) -> bool { !self.is_none() }

	#[allow(dead_code)]
	pub const fn is_single(&self) -> bool { matches!(&self, &Hint::Single(_)) }
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
