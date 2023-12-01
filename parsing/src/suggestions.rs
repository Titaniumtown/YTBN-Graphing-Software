use crate::{split_function_chars, SplitType};
use std::intrinsics::assume;

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

/// Generate a hint based on the input `input`, returns an `Option<String>`
pub fn generate_hint<'a>(input: &str) -> &'a Hint<'a> {
	if input.is_empty() {
		&HINT_EMPTY
	} else {
		let chars: Vec<char> = input.chars().collect::<Vec<char>>();

		unsafe {
			assume(!chars.is_empty());
		}

		let key = get_last_term(&chars);
		match key {
			Some(key) => {
				if let Some(hint) = COMPLETION_HASHMAP.get(&key) {
					return hint;
				}
			}
			None => {
				return &Hint::None;
			}
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

		&Hint::None
	}
}

pub fn get_last_term(chars: &[char]) -> Option<String> {
	if chars.is_empty() {
		return None;
	}

	let mut result = split_function_chars(chars, SplitType::Term);
	result.pop()
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
				write!(f, "{}", single_data)
			}
			Hint::Many(multi_data) => {
				write!(f, "{:?}", multi_data)
			}
			Hint::None => {
				write!(f, "None")
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
	#[inline]
	pub const fn is_none(&self) -> bool { matches!(&self, &Hint::None) }

	#[inline]
	#[allow(dead_code)]
	pub const fn is_some(&self) -> bool { !self.is_none() }

	#[inline]
	#[allow(dead_code)]
	pub const fn is_single(&self) -> bool { matches!(&self, &Hint::Single(_)) }

	#[inline]
	#[allow(dead_code)]
	pub const fn single(&self) -> Option<&str> {
		match self {
			Hint::Single(data) => Some(data),
			_ => None,
		}
	}

	#[inline]
	#[allow(dead_code)]
	pub const fn many(&self) -> Option<&[&str]> {
		match self {
			Hint::Many(data) => Some(data),
			_ => None,
		}
	}
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
