use std::intrinsics::assume;

use parsing::{generate_hint, Hint, HINT_EMPTY};

#[derive(PartialEq, Debug)]
pub enum Movement {
	Complete,
	Down,
	Up,
	None,
}

impl Movement {
	pub const fn is_none(&self) -> bool { matches!(&self, Self::None) }
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
