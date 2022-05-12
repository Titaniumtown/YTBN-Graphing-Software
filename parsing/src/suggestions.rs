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
		SplitType::Multiplication,
	)
	.iter()
	.map(|x| x.replace("\u{1fc93}", "exp")) // Convert back to `exp` text
	.collect::<Vec<String>>()
}

#[derive(PartialEq)]
pub enum SplitType {
	Multiplication,
	Term,
}

pub fn split_function_chars(chars: &[char], split: SplitType) -> Vec<String> {
	// Catch some basic cases
	match chars.len() {
		0 => return Vec::new(),
		1 => return vec![chars[0].to_string()],
		_ => {}
	}

	unsafe {
		assume(chars.len() > 1);
		assume(!chars.is_empty());
	}

	// Resulting split-up data
	let mut data: Vec<String> = vec![chars[0].to_string()];

	/// Used to store info about a character
	struct BoolSlice {
		closing_parens: bool,
		open_parens: bool,
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
				open_parens: *c == '(',
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

		const fn calculate_mask(&mut self, other: &BoolSlice) {
			if other.masked_num && self.number {
				// If previous char was a masked number, and current char is a number, mask current char's variable status
				self.masked_num = true;
			} else if other.masked_var && self.variable {
				// If previous char was a masked variable, and current char is a variable, mask current char's variable status
				self.masked_var = true;
			} else if other.letter && !other.is_variable() {
				// If letter and not a variable (or a masked variable)
				if self.number {
					// Mask number status if current char is number
					self.masked_num = true;
				} else if self.variable {
					// Mask variable status if current char is a variable
					self.masked_var = true;
				}
			}
		}

		const fn splitable(&self, c: &char, other: &BoolSlice, split: &SplitType) -> bool {
			if (*c == '*') | (matches!(split, &SplitType::Term) && other.open_parens) {
				return true;
			} else if other.closing_parens {
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

	// Iterate through all chars excluding the first one
	for c in chars.iter().skip(1) {
		// Set data about current character
		let mut curr_c = BoolSlice::from_char(c, prev_char.masked_num, prev_char.masked_var);

		curr_c.calculate_mask(&prev_char);

		// Append split
		if curr_c.splitable(c, &prev_char, &split) {
			data.push(String::new());
			last = unsafe { data.last_mut().unwrap_unchecked() };
		}

		// Exclude asterisks
		if c != &'*' {
			last.push(*c);
		}

		// Move current character data to `prev_char`
		prev_char = curr_c;
	}

	data
}

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint<'a>(input: &str) -> &'a Hint<'a> {
	if input.is_empty() {
		return &HINT_EMPTY;
	} else {
		let chars: Vec<char> = input.chars().collect::<Vec<char>>();

		unsafe {
			assume(!chars.is_empty());
		}

		if let Some(hint) = COMPLETION_HASHMAP.get(get_last_term(&chars).as_str()) {
			return hint;
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

		return &Hint::None;
	}
}

pub fn get_last_term(chars: &[char]) -> String {
	assert!(!chars.is_empty());

	let result = split_function_chars(chars, SplitType::Term);
	unsafe {
		assume(!result.is_empty());
		assume(result.len() > 0);
		result.last().unwrap_unchecked()
	}
	.to_owned()
}

#[derive(PartialEq, Clone, Copy)]
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
