use std::intrinsics::assume;

use crate::{generate_hint, Hint, HINT_EMPTY};

#[derive(PartialEq, Debug)]
pub enum Movement {
	Complete,
	#[allow(dead_code)]
	Down,
	#[allow(dead_code)]
	Up,
	None,
}

impl Movement {
	pub const fn is_none(&self) -> bool { matches!(&self, &Self::None) }

	pub const fn is_complete(&self) -> bool { matches!(&self, &Self::Complete) }
}

impl const Default for Movement {
	fn default() -> Self { Self::None }
}

#[derive(Clone, PartialEq)]
pub struct AutoComplete<'a> {
	pub i: usize,
	pub hint: &'a Hint<'a>,
	pub string: String,
}

impl<'a> const Default for AutoComplete<'a> {
	fn default() -> AutoComplete<'a> { AutoComplete::EMPTY }
}

impl<'a> AutoComplete<'a> {
	const EMPTY: AutoComplete<'a> = Self {
		i: 0,
		hint: &HINT_EMPTY,
		string: String::new(),
	};

	#[allow(dead_code)]
	pub fn update_string(&mut self, string: &str) {
		if self.string != string {
			// catch empty strings here to avoid call to `generate_hint` and unnecessary logic
			if string.is_empty() {
				*self = Self::EMPTY;
			} else {
				self.string = string.to_owned();
				self.do_update_logic();
			}
		}
	}

	/// Runs update logic assuming that a change to `self.string` has been made
	fn do_update_logic(&mut self) {
		self.i = 0;
		self.hint = generate_hint(&self.string);
	}

	#[allow(dead_code)]
	pub fn register_movement(&mut self, movement: &Movement) {
		if movement.is_none() {
			return;
		}

		match self.hint {
			Hint::Many(hints) => {
				// Impossible for plural hints to be singular or non-existant
				unsafe {
					assume(hints.len() > 1);
					assume(!hints.is_empty());
				}

				match movement {
					Movement::Up => {
						// if self.i is below 1, it's at
						match self.i {
							0 => self.i = hints.len() - 1,
							_ => self.i -= 1,
						}
					}
					Movement::Down => {
						// add one, if resulting value is above maximum i value, set i to 0
						self.i += 1;
						if self.i > (hints.len() - 1) {
							self.i = 0;
						}
					}
					Movement::Complete => {
						self.apply_hint(unsafe { hints.get_unchecked(self.i) });
					}
					_ => unreachable!(),
				}
			}
			Hint::Single(hint) => {
				if movement.is_complete() {
					self.apply_hint(hint);
				}
			}
			Hint::None => {}
		}
	}

	pub fn apply_hint(&mut self, hint: &str) {
		self.string.push_str(hint);
		self.do_update_logic();
	}
}
