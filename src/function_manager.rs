use crate::{
	consts::COLORS,
	function_entry::FunctionEntry,
	misc::{create_id, get_u64_id, random_u64},
	widgets::widgets_ontop,
};
use egui::{Button, Id, Key, Modifiers, TextEdit, WidgetText};
use emath::vec2;
use parsing::Movement;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::BitXorAssign;

type Functions = Vec<(Id, FunctionEntry)>;
pub struct FunctionManager {
	functions: Functions,
}

impl Default for FunctionManager {
	fn default() -> Self {
		let mut vec: Functions = Vec::with_capacity(COLORS.len());
		vec.push((
			create_id(11414819524356497634), // Random number here to avoid call to crate::misc::random_u64()
			FunctionEntry::default(),
		));
		Self { functions: vec }
	}
}

impl Serialize for FunctionManager {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut s = serializer.serialize_struct("FunctionManager", 1)?;
		s.serialize_field(
			"data",
			&self
				.functions
				.iter()
				.map(|(id, func)| (get_u64_id(*id), func.clone()))
				.collect::<Vec<(u64, FunctionEntry)>>(),
		)?;
		s.end()
	}
}

impl<'de> Deserialize<'de> for FunctionManager {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Helper(Vec<(u64, FunctionEntry)>);

		let helper = Helper::deserialize(deserializer)?;

		Ok(FunctionManager {
			functions: helper
				.0
				.iter()
				.cloned()
				.map(|(id, func)| (create_id(id), func))
				.collect::<Vec<(Id, FunctionEntry)>>(),
		})
	}
}

/// Function that creates button that's used with the `button_area`
fn button_area_button<'a>(text: impl Into<WidgetText>) -> Button<'a> {
	Button::new(text).frame(false)
}

impl FunctionManager {
	#[inline]
	fn get_hash(&self) -> u64 {
		let mut hasher = DefaultHasher::new();
		self.functions.hash(&mut hasher);
		hasher.finish()
	}

	/// Displays function entries alongside returning whether or not functions have been modified
	pub fn display_entries(&mut self, ui: &mut egui::Ui) -> bool {
		let initial_hash = self.get_hash();

		let can_remove = self.functions.len() > 1;

		let available_width = ui.available_width();
		let mut remove_i: Option<usize> = None;
		let target_size = vec2(available_width, crate::consts::FONT_SIZE);
		for (i, (te_id, function)) in self.functions.iter_mut().map(|(a, b)| (*a, b)).enumerate() {
			let mut new_string = function.autocomplete.string.clone();
			function.update_string(&new_string);

			let mut movement: Movement = Movement::default();

			let size_multiplier = vec2(1.0, {
				let had_focus = ui.memory(|x| x.has_focus(te_id));
				(ui.ctx().animate_bool(te_id, had_focus) * 1.5) + 1.0
			});

			let re = ui.add_sized(
				target_size * size_multiplier,
				egui::TextEdit::singleline(&mut new_string)
					.hint_forward(true) // Make the hint appear after the last text in the textbox
					.lock_focus(true)
					.id(te_id) // Set widget's id to `te_id`
					.hint_text(
						// If there's a single hint, go ahead and apply the hint here, if not, set the hint to an empty string
						function.autocomplete.hint.single().unwrap_or(""),
					),
			);

			// Only keep valid chars
			new_string.retain(crate::misc::is_valid_char);

			// If not fully open, return here as buttons cannot yet be displayed, therefore the user is inable to mark it for deletion
			let animate_bool = ui.ctx().animate_bool(te_id, re.has_focus());
			if animate_bool == 1.0 {
				function.autocomplete.update_string(&new_string);

				if function.autocomplete.hint.is_some() {
					// only register up and down arrow movements if hint is type `Hint::Many`
					if !function.autocomplete.hint.is_single() {
						let (arrow_down, arrow_up) = ui.input(|x| {
							(x.key_pressed(Key::ArrowDown), x.key_pressed(Key::ArrowUp))
						});
						if arrow_down {
							movement = Movement::Down;
						} else if arrow_up {
							movement = Movement::Up;
						}
					}

					// Put here so these key presses don't interact with other elements
					let movement_complete_action = ui.input_mut(|x| {
						x.consume_key(Modifiers::NONE, Key::Enter)
							| x.consume_key(Modifiers::NONE, Key::Tab)
							| x.key_pressed(Key::ArrowRight)
					});

					if movement_complete_action {
						movement = Movement::Complete;
					}

					// Register movement and apply proper changes
					function.autocomplete.register_movement(&movement);

					if movement != Movement::Complete
						&& let Some(hints) = function.autocomplete.hint.many()
					{
						// Doesn't need to have a number in id as there should only be 1 autocomplete popup in the entire gui

						// hashed "autocomplete_popup"
						const POPUP_ID: Id = create_id(7574801616484505465);

						let mut clicked = false;

						egui::popup_below_widget(ui, POPUP_ID, &re, |ui| {
							hints.iter().enumerate().for_each(|(i, candidate)| {
								if ui
									.selectable_label(i == function.autocomplete.i, *candidate)
									.clicked()
								{
									clicked = true;
									function.autocomplete.i = i;
								}
							});
						});

						if clicked {
							function
								.autocomplete
								.apply_hint(hints[function.autocomplete.i]);

							// Don't need this here as it simply won't be display next frame
							// ui.memory_mut().close_popup();

							movement = Movement::Complete;
						} else {
							ui.memory_mut(|x| x.open_popup(POPUP_ID));
						}
					}

					// Push cursor to end if needed
					if movement == Movement::Complete {
						let mut state =
							unsafe { TextEdit::load_state(ui.ctx(), te_id).unwrap_unchecked() };
						let ccursor = egui::text::CCursor::new(function.autocomplete.string.len());
						state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
						TextEdit::store_state(ui.ctx(), te_id, state);
					}
				}

				/// The y offset multiplier of the `buttons_area` area
				const BUTTONS_Y_OFFSET: f32 = 1.32;
				const Y_OFFSET: f32 = crate::consts::FONT_SIZE * BUTTONS_Y_OFFSET;

				widgets_ontop(ui, create_id(i as u64), &re, Y_OFFSET, |ui| {
					ui.horizontal(|ui| {
						// There's more than 1 function! Functions can now be deleted
						if ui
							.add_enabled(can_remove, button_area_button("✖"))
							.on_hover_text("Delete Function")
							.clicked()
						{
							remove_i = Some(i);
						}

						ui.add_enabled_ui(function.is_some(), |ui| {
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

							// Toggle showing the settings window
							function.settings_opened.bitxor_assign(
								ui.add(button_area_button("⚙"))
									.on_hover_text(match function.settings_opened {
										true => "Close Settings",
										false => "Open Settings",
									})
									.clicked(),
							);
						});
					});
				});
			}

			function.settings_window(ui.ctx());
		}

		// Remove function if the user requests it
		if let Some(remove_i_unwrap) = remove_i {
			self.functions.remove(remove_i_unwrap);
		}

		let final_hash = self.get_hash();

		initial_hash != final_hash
	}

	/// Create and push new empty function entry
	pub fn push_empty(&mut self) {
		self.functions.push((
			create_id(random_u64().expect("unable to generate random id")),
			FunctionEntry::default(),
		));
	}

	/// Detect if any functions are using integrals
	pub fn any_using_integral(&self) -> bool {
		self.functions.iter().any(|(_, func)| func.integral)
	}

	#[inline]
	pub fn len(&self) -> usize { self.functions.len() }

	#[inline]
	pub fn get_entries_mut(&mut self) -> &mut Functions { &mut self.functions }

	#[inline]
	pub fn get_entries(&self) -> &Functions { &self.functions }
}
