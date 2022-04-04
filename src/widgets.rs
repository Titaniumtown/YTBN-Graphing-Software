use crate::suggestions::{generate_hint, HintEnum};
use eframe::{egui, epaint};
use egui::{text::CCursor, text_edit::CursorRange, Key, Modifiers, TextEdit, Widget};
use epaint::text::cursor::{Cursor, PCursor, RCursor};

#[derive(Clone)]
pub struct AutoComplete {
	pub i: usize,
	pub hint: HintEnum<'static>,
	pub func_str: Option<String>,
	pub changed: bool,
}

impl Default for AutoComplete {
	fn default() -> AutoComplete {
		AutoComplete {
			i: 0,
			hint: HintEnum::None,
			func_str: None,
			changed: true,
		}
	}
}

impl AutoComplete {
	fn changed(&mut self, string: &str) {
		if self.func_str != Some(string.to_string()) {
			self.changed = true;
			self.func_str = Some(string.to_string());
			self.hint = generate_hint(string);
		} else {
			self.changed = false;
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, string: String, func_i: i32) -> (String, bool) {
		let mut new_string = string.clone();
		// Put here so these key presses don't interact with other elements
		let enter_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Enter);
		let tab_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Tab);

		let te_id = ui.make_persistent_id(format!("text_edit_ac_{}", func_i));

		// update self
		self.changed(&string);

		let mut func_edit = egui::TextEdit::singleline(&mut new_string)
			.hint_forward(true) // Make the hint appear after the last text in the textbox
			.lock_focus(true);

		if self.hint.is_none() {
			let re = func_edit.id(te_id).ui(ui);
			let return_string = (&new_string).to_string();
			return (return_string, re.has_focus());
		}

		if let Some(single_hint) = self.hint.get_single() {
			let func_edit_2 = func_edit;
			func_edit = func_edit_2.hint_text(&single_hint);
		}

		let re = func_edit.id(te_id).ui(ui);

		let func_edit_focus = re.has_focus();

		// If in focus and right arrow key was pressed, apply hint
		if func_edit_focus {
			let mut push_cursor: bool = false;
			let apply_key = ui.input().key_pressed(Key::ArrowRight) | enter_pressed | tab_pressed;

			if apply_key && let Some(single_hint) = self.hint.get_single() {
					push_cursor = true;
					new_string = string + &single_hint;
			} else if self.hint.is_multi() {
				let selections = self.hint.ensure_many();

				let max_i = selections.len() as i16 - 1;

				let mut i = self.i as i16;

				if ui.input().key_pressed(Key::ArrowDown) {
					i += 1;
					if i > max_i {
						i = 0;
					}
				} else if ui.input().key_pressed(Key::ArrowUp) {
					i -= 1;
					if 0 > i {
						i = max_i
					}
				}

				self.i = i as usize;

				// Doesn't need to have a number in id as there should only be 1 autocomplete popup in entire gui
				let popup_id = ui.make_persistent_id("autocomplete_popup");

				let mut clicked = false;

				egui::popup_below_widget(ui, popup_id, &re, |ui| {
					for (i, candidate) in selections.iter().enumerate() {
						if ui
							.selectable_label(i == self.i, *candidate)
							.clicked()
						{
							clicked = true;
							self.i = i;
						}
					}
				});

				if clicked | apply_key {
					new_string += selections[self.i];
					push_cursor = true;


					// don't need this here as it simply won't be display next frame in `math_app.rs`
					// ui.memory().close_popup();
				} else {
					ui.memory().open_popup(popup_id);
				}
			}

			// Push cursor to end if needed
			if push_cursor {
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
