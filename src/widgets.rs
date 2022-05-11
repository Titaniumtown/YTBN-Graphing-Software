use std::intrinsics::assume;

use parsing::suggestions::{self, generate_hint, Hint};

#[derive(PartialEq, Debug)]
pub enum Movement {
	Complete,
	Down,
	Up,
	None,
}

impl const Default for Movement {
	fn default() -> Self { Self::None }
}

#[derive(Clone)]
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
		hint: &suggestions::HINT_EMPTY,
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

	pub fn register_movement(&mut self, movement: &Movement) {
		if movement == &Movement::None {
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
						// subtract one, if fail, set to maximum index value.
						self.i = self.i.checked_sub(1).unwrap_or(hints.len() - 1);
					}
					Movement::Down => {
						// add one, if resulting value is above maximum i value, set i to 0
						self.i += 1;
						if self.i > (hints.len() - 1) {
							self.i = 0;
						}
					}
					Movement::Complete => {
						unsafe { assume(hints.len() >= (self.i + 1)) }

						self.apply_hint(hints[self.i]);
					}
					Movement::None => {}
				}
			}
			Hint::Single(hint) => {
				if movement == &Movement::Complete {
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

pub fn widgets_ontop<R>(
	ui: &egui::Ui, id: String, re: &egui::Response, y_offset: f32,
	add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
	let area = egui::Area::new(id)
		.fixed_pos(re.rect.min.offset_y(y_offset))
		.order(egui::Order::Foreground);

	area.show(ui.ctx(), |ui| add_contents(ui)).inner
}

#[cfg(test)]
mod autocomplete_tests {
	use super::*;

	enum Action<'a> {
		AssertIndex(usize),
		AssertString(&'a str),
		AssertHint(&'a str),
		SetString(&'a str),
		Move(Movement),
	}

	fn ac_tester(actions: &[Action]) {
		let mut ac = AutoComplete::default();
		for action in actions.iter() {
			match action {
				Action::AssertIndex(target_i) => {
					if &ac.i != target_i {
						panic!(
							"AssertIndex failed: Current: '{}' Expected: '{}'",
							ac.i, target_i
						)
					}
				}
				Action::AssertString(target_string) => {
					if &ac.string != target_string {
						panic!(
							"AssertString failed: Current: '{}' Expected: '{}'",
							ac.string, target_string
						)
					}
				}
				Action::AssertHint(target_hint) => match ac.hint {
					Hint::None => {
						if !target_hint.is_empty() {
							panic!(
								"AssertHint failed on `Hint::None`: Expected: {}",
								target_hint
							);
						}
					}
					Hint::Many(hints) => {
						let hint = hints[ac.i];
						if &hint != target_hint {
							panic!(
								"AssertHint failed on `Hint::Many`: Current: '{}' (index: {}) Expected: '{}'",
								hint, ac.i, target_hint
							)
						}
					}
					Hint::Single(hint) => {
						if hint != target_hint {
							panic!(
								"AssertHint failed on `Hint::Single`: Current: '{}' Expected: '{}'",
								hint, target_hint
							)
						}
					}
				},
				Action::SetString(target_string) => {
					ac.update_string(target_string);
				}
				Action::Move(target_movement) => {
					ac.register_movement(target_movement);
				}
			}
		}
	}

	#[test]
	fn single() {
		ac_tester(&[
			Action::SetString(""),
			Action::AssertHint("x^2"),
			Action::Move(Movement::Up),
			Action::AssertIndex(0),
			Action::AssertString(""),
			Action::AssertHint("x^2"),
			Action::Move(Movement::Down),
			Action::AssertIndex(0),
			Action::AssertString(""),
			Action::AssertHint("x^2"),
			Action::Move(Movement::Complete),
			Action::AssertString("x^2"),
			Action::AssertHint(""),
			Action::AssertIndex(0),
		]);
	}

	#[test]
	fn multi() {
		ac_tester(&[
			Action::SetString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Up),
			Action::AssertIndex(3),
			Action::AssertString("s"),
			Action::AssertHint("ignum("),
			Action::Move(Movement::Down),
			Action::AssertIndex(0),
			Action::AssertString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Down),
			Action::AssertIndex(1),
			Action::AssertString("s"),
			Action::AssertHint("qrt("),
			Action::Move(Movement::Up),
			Action::AssertIndex(0),
			Action::AssertString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Complete),
			Action::AssertString("sin("),
			Action::AssertHint(")"),
			Action::AssertIndex(0),
		]);
	}

	#[test]
	fn parens() {
		ac_tester(&[
			Action::SetString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Up),
			Action::AssertIndex(0),
			Action::AssertString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Down),
			Action::AssertIndex(0),
			Action::AssertString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Complete),
			Action::AssertString("sin(x)"),
			Action::AssertHint(""),
			Action::AssertIndex(0),
		]);
	}
}
