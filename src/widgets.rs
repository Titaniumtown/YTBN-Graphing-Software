use crate::function_handling::suggestions::{self, generate_hint, Hint};

use eframe::{egui, epaint};
use egui::{text::CCursor, text_edit::CursorRange, TextEdit};
use epaint::text::cursor::{Cursor, PCursor, RCursor};

#[derive(PartialEq, Debug)]
pub enum Movement {
	Complete,
	Down,
	Up,
	None,
}

impl Default for Movement {
	fn default() -> Self { Self::None }
}

#[derive(Clone)]
pub struct AutoComplete<'a> {
	pub i: usize,
	pub hint: &'a Hint<'a>,
	pub string: String,
}

impl<'a> Default for AutoComplete<'a> {
	fn default() -> AutoComplete<'a> {
		AutoComplete {
			i: 0,
			hint: &suggestions::HINT_EMPTY,
			string: String::new(),
		}
	}
}

impl<'a> AutoComplete<'a> {
	pub fn update_string(&mut self, string: &str) {
		if self.string != string {
			self.i = 0;
			self.string = string.to_string();
			self.hint = generate_hint(string);
		}
	}

	pub fn register_movement(&mut self, movement: &Movement) {
		if movement == &Movement::None {
			return;
		}

		match self.hint {
			Hint::Many(hints) => {
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
		let new_string = self.string.clone() + hint;
		self.update_string(&new_string);
	}
}

/// Moves cursor of TextEdit `te_id` to the end
pub fn move_cursor_to_end(ctx: &egui::Context, te_id: egui::Id) {
	let mut state = TextEdit::load_state(ctx, te_id).expect("Expected TextEdit");
	state.set_cursor_range(Some(CursorRange::one(Cursor {
		ccursor: CCursor {
			index: 0,
			prefer_next_row: false,
		},
		rcursor: RCursor { row: 0, column: 0 },
		pcursor: PCursor {
			paragraph: 0,
			offset: 10000,
			prefer_next_row: false,
		},
	})));
	TextEdit::store_state(ctx, te_id, state);
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
