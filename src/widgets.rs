use crate::suggestions::{generate_hint, HintEnum};
use eframe::{egui, epaint};
use egui::{text::CCursor, text_edit::CursorRange, Key, Modifiers, TextEdit, Widget};
use epaint::text::cursor::{Cursor, PCursor, RCursor};

#[derive(PartialEq, Debug)]
enum Movement {
	Complete,
	Down,
	Up,
	None,
}

#[derive(Clone)]
pub struct AutoComplete<'a> {
	pub i: usize,
	pub hint: &'a HintEnum<'a>,
	pub string: String,
}

impl Default for Movement {
	fn default() -> Self { Self::None }
}

impl<'a> Default for AutoComplete<'a> {
	fn default() -> AutoComplete<'a> {
		AutoComplete {
			i: 0,
			hint: &crate::suggestions::HINTENUM_EMPTY,
			string: String::new(),
		}
	}
}

impl<'a> AutoComplete<'a> {
	pub fn update(&mut self, string: &str) {
		if &self.string != string {
			self.i = 0;
			self.string = string.to_string();
			self.hint = generate_hint(string);
		}
	}

	fn interact_back(&mut self, movement: &Movement) {
		if movement == &Movement::None {
			return;
		}

		match self.hint {
			HintEnum::Many(hints) => {
				if movement == &Movement::Complete {
					self.apply_hint(hints[self.i]);
					return;
				}

				// maximum index value
				let max_i = hints.len() - 1;

				match movement {
					Movement::Up => {
						// subtract one, if fail, set to max_i value.
						self.i = self.i.checked_sub(1).unwrap_or(max_i);
					}
					Movement::Down => {
						// add one, if resulting value is above maximum i value, set i to 0
						self.i += 1;
						if self.i > max_i {
							self.i = 0;
						}
					}
					_ => unreachable!(),
				}
			}
			HintEnum::Single(hint) => {
				if movement == &Movement::Complete {
					self.apply_hint(hint);
				}
			}
			HintEnum::None => {}
		}
	}

	fn apply_hint(&mut self, hint: &str) {
		let new_string = self.string.clone() + hint;
		self.update(&new_string);
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, func_i: i32) {
		let mut movement: Movement = Movement::default();

		let mut new_string = self.string.clone();

		let te_id = ui.make_persistent_id(format!("text_edit_ac_{}", func_i));

		let mut func_edit = egui::TextEdit::singleline(&mut new_string)
			.hint_forward(true) // Make the hint appear after the last text in the textbox
			.lock_focus(true)
			.id(te_id);

		if self.hint.is_none() {
			let _ = func_edit.ui(ui);
			self.update(&new_string);
			return;
		}

		// Put here so these key presses don't interact with other elements
		let enter_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Enter);
		let tab_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Tab);
		if enter_pressed | tab_pressed | ui.input().key_pressed(Key::ArrowRight) {
			movement = Movement::Complete;
		}

		if let HintEnum::Single(single_hint) = self.hint {
			func_edit = func_edit.hint_text(*single_hint);
		}

		let re = func_edit.ui(ui);

		self.update(&new_string);

		if !self.hint.is_single() {
			if ui.input().key_pressed(Key::ArrowDown) {
				movement = Movement::Down;
			} else if ui.input().key_pressed(Key::ArrowUp) {
				movement = Movement::Up;
			}
		}

		self.interact_back(&movement);

		if movement != Movement::Complete && let HintEnum::Many(hints) = self.hint {
			// Doesn't need to have a number in id as there should only be 1 autocomplete popup in the entire gui
			let popup_id = ui.make_persistent_id("autocomplete_popup");

			let mut clicked = false;

			egui::popup_below_widget(ui, popup_id, &re, |ui| {
				hints.iter().enumerate().for_each(|(i, candidate)| {
					if ui.selectable_label(i == self.i, *candidate).clicked() {
						clicked = true;
						self.i = i;
					}
				});
			});

			if clicked {
				self.apply_hint(hints[self.i]);

				// don't need this here as it simply won't be display next frame
				// ui.memory().close_popup();

				movement = Movement::Complete;
			} else {
				ui.memory().open_popup(popup_id);
			}
		}

		// Push cursor to end if needed
		if movement == Movement::Complete {
			let mut state = TextEdit::load_state(ui.ctx(), te_id).unwrap();
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
			TextEdit::store_state(ui.ctx(), te_id, state);
		}
	}
}

#[cfg(test)]
mod autocomplete_tests {
	use super::*;

	enum Action<'a> {
		AssertI(usize),
		AssertString(&'a str),
		AssertHint(&'a str),
		SetString(&'a str),
		Move(Movement),
	}

	fn ac_tester(actions: &[Action]) {
		let mut ac = AutoComplete::default();
		for action in actions.iter() {
			match action {
				Action::AssertI(target_i) => {
					if &ac.i != target_i {
						panic!(
							"AssertI failed: Current: '{}' Expected: '{}'",
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
					HintEnum::None => {
						if !target_hint.is_empty() {
							panic!(
								"AssertHint failed on `HintEnum::None`: Expected: {}",
								target_hint
							);
						}
					}
					HintEnum::Many(hints) => {
						let hint = hints[ac.i];
						if &hint != target_hint {
							panic!(
								"AssertHint failed on `HintEnum::Many`: Current: '{}' (index: {}) Expected: '{}'",
								hint, ac.i, target_hint
							)
						}
					}
					HintEnum::Single(hint) => {
						if hint != target_hint {
							panic!(
								"AssertHint failed on `HintEnum::Single`: Current: '{}' Expected: '{}'",
								hint, target_hint
							)
						}
					}
				},
				Action::SetString(target_string) => {
					ac.update(target_string);
				}
				Action::Move(target_movement) => {
					ac.interact_back(target_movement);
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
			Action::AssertI(0),
			Action::AssertString(""),
			Action::AssertHint("x^2"),
			Action::Move(Movement::Down),
			Action::AssertI(0),
			Action::AssertString(""),
			Action::AssertHint("x^2"),
			Action::Move(Movement::Complete),
			Action::AssertString("x^2"),
			Action::AssertHint(""),
			Action::AssertI(0),
		]);
	}

	#[test]
	fn multi() {
		ac_tester(&[
			Action::SetString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Up),
			Action::AssertI(3),
			Action::AssertString("s"),
			Action::AssertHint("ignum("),
			Action::Move(Movement::Down),
			Action::AssertI(0),
			Action::AssertString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Down),
			Action::AssertI(1),
			Action::AssertString("s"),
			Action::AssertHint("qrt("),
			Action::Move(Movement::Up),
			Action::AssertI(0),
			Action::AssertString("s"),
			Action::AssertHint("in("),
			Action::Move(Movement::Complete),
			Action::AssertString("sin("),
			Action::AssertHint(")"),
			Action::AssertI(0),
		]);
	}

	#[test]
	fn parens() {
		ac_tester(&[
			Action::SetString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Up),
			Action::AssertI(0),
			Action::AssertString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Down),
			Action::AssertI(0),
			Action::AssertString("sin(x"),
			Action::AssertHint(")"),
			Action::Move(Movement::Complete),
			Action::AssertString("sin(x)"),
			Action::AssertHint(""),
			Action::AssertI(0),
		]);
	}
}
