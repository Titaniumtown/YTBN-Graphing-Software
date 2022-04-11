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
	pub string: Option<String>,
}

impl Default for Movement {
	fn default() -> Movement { Movement::None }
}

impl<'a> Default for AutoComplete<'a> {
	fn default() -> AutoComplete<'a> {
		AutoComplete {
			i: 0,
			hint: &HintEnum::None,
			string: None,
		}
	}
}

impl<'a> AutoComplete<'a> {
	fn update(&mut self, string: &str) {
		let new_func_option = Some(string.to_string());
		if self.string != new_func_option {
			self.string = new_func_option;
			self.hint = generate_hint(string);
		}
	}

	fn interact_back(&mut self, movement: &Movement) {
		if movement == &Movement::None {
			return;
		}

		// self.string needs to be Some for this to work __DO NOT REMOVE THIS ASSERT__
		assert!(self.string.is_some());

		match self.hint {
			HintEnum::Many(hints) => {
				if movement == &Movement::Complete {
					// use unwrap_unchecked as self.string is already asserted as Some
					unsafe {
						*self.string.as_mut().unwrap_unchecked() += hints[self.i];
					}
					return;
				}

				// maximum i value
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
					// use unwrap_unchecked as self.string is already asserted as Some
					unsafe {
						*self.string.as_mut().unwrap_unchecked() += hint;
					}
				}
			}
			HintEnum::None => {}
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, string: &mut String, func_i: i32) {
		let mut movement: Movement = Movement::default();

		// update self
		self.update(string);

		let mut func_edit = egui::TextEdit::singleline(string)
			.hint_forward(true) // Make the hint appear after the last text in the textbox
			.lock_focus(true);

		let te_id = ui.make_persistent_id(format!("text_edit_ac_{}", func_i));

		if self.hint.is_none() {
			let _ = func_edit.id(te_id).ui(ui);
			return;
		}

		// Put here so these key presses don't interact with other elements
		let enter_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Enter);
		let tab_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Tab);
		if enter_pressed | tab_pressed | ui.input().key_pressed(Key::ArrowRight) {
			println!("complete");
			movement = Movement::Complete;
		}

		if let HintEnum::Single(single_hint) = self.hint {
			func_edit = func_edit.hint_text(*single_hint);
		}

		let re = func_edit.id(te_id).ui(ui);

		if ui.input().key_pressed(Key::ArrowDown) {
			movement = Movement::Down;
		} else if ui.input().key_pressed(Key::ArrowUp) {
			movement = Movement::Up;
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
				*string += hints[self.i];

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
mod tests {
	use super::*;

	fn auto_complete_helper(string: &str, movement: Movement) -> (AutoComplete, String) {
		let mut auto_complete = AutoComplete::default();
		auto_complete.update(string);
		auto_complete.interact_back(&movement);

		let output_string = auto_complete.clone().string;
		(auto_complete, output_string.unwrap_or_default())
	}

	#[test]
	fn auto_complete_single_still() {
		let (_, string) = auto_complete_helper("", Movement::None);

		assert_eq!(&*string, "");
	}

	#[test]
	fn auto_complete_single_complete() {
		let (_, string) = auto_complete_helper("", Movement::Complete);

		assert_eq!(&*string, "x^2");
	}

	#[test]
	fn auto_complete_single_down() {
		let (_, string) = auto_complete_helper("", Movement::Down);

		assert_eq!(&*string, "");
	}

	#[test]
	fn auto_complete_single_up() {
		let (_, string) = auto_complete_helper("", Movement::Up);

		assert_eq!(&*string, "");
	}

	#[test]
	fn auto_complete_multi_up() {
		let (auto_complete, string) = auto_complete_helper("s", Movement::Up);

		assert!(auto_complete.i > 0);
		assert_eq!(&*string, "s");
	}

	#[test]
	fn auto_complete_multi_down() {
		let (auto_complete, string) = auto_complete_helper("s", Movement::Down);

		assert_eq!(auto_complete.i, 1);
		assert_eq!(&*string, "s");
	}

	#[test]
	fn auto_complete_multi_complete() {
		let (auto_complete, string) = auto_complete_helper("s", Movement::Complete);

		assert_eq!(auto_complete.i, 0);
		assert_eq!(&*string, "sin(");
	}
}
