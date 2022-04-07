use crate::suggestions::{generate_hint, HintEnum};
use eframe::{egui, epaint};
use egui::{text::CCursor, text_edit::CursorRange, Key, Label, Modifiers, TextEdit, Widget};
use epaint::text::cursor::{Cursor, PCursor, RCursor};

#[derive(Clone)]
pub struct AutoComplete<'a> {
	pub i: usize,
	pub hint: &'a HintEnum<'a>,
	pub func_str: Option<String>,
	pub changed: bool,
}

#[derive(PartialEq, Debug)]
enum Movement {
	Complete,
	Down,
	Up,
	None,
}

impl<'a> Default for AutoComplete<'a> {
	fn default() -> AutoComplete<'a> {
		AutoComplete {
			i: 0,
			hint: &HintEnum::None,
			func_str: None,
			changed: true,
		}
	}
}

impl<'a> AutoComplete<'a> {
	fn changed(&mut self, string: &str) {
		let new_func_str = Some(string.to_string());
		if self.func_str != new_func_str {
			self.changed = true;
			self.func_str = new_func_str;
			self.hint = generate_hint(string);
		} else {
			self.changed = false;
		}
	}

	fn interact_back(&mut self, new_string: &mut String, movement: &Movement) {
		match self.hint {
			HintEnum::Many(hints) => {
				if movement == &Movement::Complete {
					*new_string += hints[self.i];
					return;
				} else if movement == &Movement::None {
					return;
				}

				let max_i = hints.len() as i16 - 1;
				let mut i = self.i as i16;

				match movement {
					Movement::Up => {
						i -= 1;
						if 0 > i {
							i = max_i
						}
					}
					Movement::Down => {
						i += 1;
						if i > max_i {
							i = 0;
						}
					}
					_ => {}
				}
				self.i = i as usize;
			}
			HintEnum::Single(hint) => match movement {
				Movement::Complete => {
					*new_string += hint;
				}
				_ => {}
			},
			HintEnum::None => {}
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, string: String, func_i: i32) -> (String, bool) {
		let mut new_string = string.clone();

		let mut movement: Movement = Movement::None;

		// update self
		self.changed(&string);

		let mut func_edit = egui::TextEdit::singleline(&mut new_string)
			.hint_forward(true) // Make the hint appear after the last text in the textbox
			.lock_focus(true);

		let te_id = ui.make_persistent_id(format!("text_edit_ac_{}", func_i));

		if self.hint.is_none() {
			let re = func_edit.id(te_id).ui(ui);
			let return_string = (&new_string).to_string();
			return (return_string, re.has_focus());
		}

		// Put here so these key presses don't interact with other elements
		let enter_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Enter);
		let tab_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Tab);
		if enter_pressed | tab_pressed | ui.input().key_pressed(Key::ArrowRight) {
			movement = Movement::Complete;
		}

		if let HintEnum::Single(single_hint) = self.hint {
			let func_edit_2 = func_edit;
			func_edit = func_edit_2.hint_text(*single_hint);
		}

		let re = func_edit.id(te_id).ui(ui);

		let func_edit_focus = re.has_focus();

		// If in focus and right arrow key was pressed, apply hint
		if func_edit_focus {
			if ui.input().key_pressed(Key::ArrowDown) {
				movement = Movement::Down;
			} else if ui.input().key_pressed(Key::ArrowUp) {
				movement = Movement::Up;
			}

			// if movement != Movement::None {
			// 	println!("{:?}", movement);
			// }

			self.interact_back(&mut new_string, &movement);

			// TODO: fix clicking on labels (no clue why it doesn't work, time to take a walk)
			if movement != Movement::Complete && let HintEnum::Many(hints) = self.hint {
				// Doesn't need to have a number in id as there should only be 1 autocomplete popup in the entire gui
				let popup_id = ui.make_persistent_id("autocomplete_popup");

				// let mut clicked = false;

				egui::popup_below_widget(ui, popup_id, &re, |ui| {
					hints.iter().enumerate().for_each(|(i, candidate)| {
						/*
						if ui.selectable_label(i == self.i, *candidate).clicked() {
							clicked = true;
							self.i = i;
						}
						*/

						// placeholder for now
						ui.add_enabled(i == self.i, Label::new(*candidate));
					});
				});

				ui.memory().open_popup(popup_id)
				/*
				if clicked {
					new_string += hints[self.i];

					// don't need this here as it simply won't be display next frame in `math_app.rs`
					// ui.memory().close_popup();

					movement = Movement::Complete;
				} else {
					ui.memory().open_popup(popup_id);
				}
				*/
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
		(new_string, func_edit_focus)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn auto_complete_helper(string: &str, movement: Movement) -> (AutoComplete, String) {
		let mut auto_complete = AutoComplete::default();
		auto_complete.changed(string);
		let mut string_1 = String::from(string);
		auto_complete.interact_back(&mut string_1, &movement);

		(auto_complete, string_1)
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
