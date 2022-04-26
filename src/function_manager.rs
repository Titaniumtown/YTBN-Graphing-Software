use crate::consts::is_mobile;
use crate::function_entry::{FunctionEntry, DEFAULT_FUNCTION_ENTRY};
use crate::widgets::{move_cursor_to_end, widgets_ontop, Movement};
use egui::{Button, Key, Modifiers};
use emath::vec2;
use parsing::suggestions::Hint;
use std::ops::BitXorAssign;
use uuid::Uuid;

pub struct FunctionManager {
	functions: Vec<(Uuid, FunctionEntry)>,
}

impl Default for FunctionManager {
	fn default() -> Self {
		Self {
			functions: vec![(Uuid::new_v4(), DEFAULT_FUNCTION_ENTRY.clone())],
		}
	}
}

impl FunctionManager {
	pub fn display_entries(&mut self, ui: &mut egui::Ui) {
		// ui.label("Functions:");
		let can_remove = self.functions.len() > 1;

		let row_height = ui
			.fonts()
			.row_height(&egui::FontSelection::default().resolve(ui.style()));

		let mut remove_i: Option<usize> = None;
		for (i, (uuid, function)) in self.functions.iter_mut().enumerate() {
			// render cool input box
			let output_string = function.autocomplete.string.clone();
			function.update_string(&output_string);

			let mut movement: Movement = Movement::default();

			let mut new_string = function.autocomplete.string.clone();

			// unique id for this function
			let te_id = ui.make_persistent_id(uuid);

			// target size of text edit box
			let target_size = vec2(ui.available_width(), {
				// get the animated bool that stores how "in focus" the text box is
				let gotten_focus_value = {
					let ctx = ui.ctx();
					let had_focus = ctx.memory().has_focus(te_id);
					ctx.animate_bool(te_id, had_focus)
				};

				row_height * (1.0 + (gotten_focus_value * 1.5))
			});

			let re = ui.add_sized(
				target_size,
				egui::TextEdit::singleline(&mut new_string)
					.hint_forward(true) // Make the hint appear after the last text in the textbox
					.lock_focus(true)
					.id(te_id) // set widget's id to `te_id`
					.hint_text({
						// if there's a single hint, go ahead and apply the hint here, if not, set the hint to an empty string
						if let Hint::Single(single_hint) = function.autocomplete.hint {
							*single_hint
						} else {
							""
						}
					}),
			);

			// if not fully open, return here as buttons cannot yet be displayed, therefore the user is inable to mark it for deletion
			if ui.ctx().animate_bool(te_id, re.has_focus()) < 1.0 {
				continue;
			}

			function.autocomplete.update_string(&new_string);

			if !function.autocomplete.hint.is_none() {
				if !is_mobile() && !function.autocomplete.hint.is_single() {
					if ui.input().key_pressed(Key::ArrowDown) {
						movement = Movement::Down;
					} else if ui.input().key_pressed(Key::ArrowUp) {
						movement = Movement::Up;
					}
				}

				// Put here so these key presses don't interact with other elements
				let enter_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Enter);
				let tab_pressed = ui.input_mut().consume_key(Modifiers::NONE, Key::Tab);
				if enter_pressed | tab_pressed | ui.input().key_pressed(Key::ArrowRight) {
					movement = Movement::Complete;
				}

				function.autocomplete.register_movement(&movement);

				if movement != Movement::Complete && let Hint::Many(hints) = function.autocomplete.hint {
                    // Doesn't need to have a number in id as there should only be 1 autocomplete popup in the entire gui
                    let popup_id = ui.make_persistent_id("autocomplete_popup");

                    let mut clicked = false;

                    egui::popup_below_widget(ui, popup_id, &re, |ui| {
                        hints.iter().enumerate().for_each(|(i, candidate)| {
                            if ui.selectable_label(i == function.autocomplete.i, *candidate).clicked() {
                                clicked = true;
                                function.autocomplete.i = i;
                            }
                        });
                    });

                    if clicked {
                        function.autocomplete.apply_hint(hints[function.autocomplete.i]);

                        // don't need this here as it simply won't be display next frame
                        // ui.memory().close_popup();

                        movement = Movement::Complete;
                    } else {
                        ui.memory().open_popup(popup_id);
                    }
                }

				// Push cursor to end if needed
				if movement == Movement::Complete {
					move_cursor_to_end(ui.ctx(), te_id);
				}
			}

			/// Function that creates button that's used with the `button_area`
			fn button_area_button(text: impl Into<egui::WidgetText>) -> Button {
				Button::new(text.into()).frame(false)
			}

			/// the y offset multiplier of the `buttons_area` area
			const BUTTONS_Y_OFFSET: f32 = 1.32;

			widgets_ontop(
				ui,
				format!("buttons_area_{}", i),
				&re,
				row_height * BUTTONS_Y_OFFSET,
				|ui| {
					ui.horizontal(|ui| {
						// There's more than 1 function! Functions can now be deleted
						if ui
							.add_enabled(can_remove, button_area_button("✖"))
							.on_hover_text("Delete Function")
							.clicked()
						{
							remove_i = Some(i);
						}

						// Toggle integral being enabled or not
						function.integral.bitxor_assign(
							ui.add(button_area_button("∫"))
								.on_hover_text(match function.integral {
									true => "Don't integrate",
									false => "Integrate",
								})
								.clicked(),
						);

						// Toggle showing the derivative (even though it's already calculated this option just toggles if it's displayed or not)
						function.derivative.bitxor_assign(
							ui.add(button_area_button("d/dx"))
								.on_hover_text(match function.derivative {
									true => "Don't Differentiate",
									false => "Differentiate",
								})
								.clicked(),
						);

						function.settings_opened.bitxor_assign(
							ui.add(button_area_button("⚙"))
								.on_hover_text(match function.settings_opened {
									true => "Close Settings",
									false => "Open Settings",
								})
								.clicked(),
						);
					});
				},
			);

			function.settings_window(ui.ctx());
		}

		// Remove function if the user requests it
		if let Some(remove_i_unwrap) = remove_i {
			self.functions.remove(remove_i_unwrap);
		}
	}

	pub fn new_function(&mut self) {
		self.functions
			.push((Uuid::new_v4(), DEFAULT_FUNCTION_ENTRY.clone()));
	}

	pub fn any_using_integral(&self) -> bool {
		self.functions
			.iter()
			.filter(|(_, func)| func.integral)
			.count() > 0
	}

	#[inline]
	pub fn len(&self) -> usize { self.functions.len() }

	pub fn get_entries_mut(&mut self) -> &mut Vec<(Uuid, FunctionEntry)> { &mut self.functions }

	pub fn get_entries(&self) -> &Vec<(Uuid, FunctionEntry)> { &self.functions }
}
