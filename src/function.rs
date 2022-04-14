#![allow(clippy::too_many_arguments)] // Clippy, shut

use crate::math_app::AppSettings;
use crate::misc::*;
use crate::parsing::{process_func_str, BackingFunction};
use crate::suggestions::Hint;
use crate::widgets::{AutoComplete, Movement};
use eframe::{egui, emath, epaint};
use egui::{
	plot::{BarChart, PlotUi, Value},
	text::CCursor,
	text_edit::CursorRange,
	widgets::plot::Bar,
	Button, Checkbox, Context, Key, Modifiers, TextEdit,
};
use emath::{pos2, vec2};
use epaint::{
	text::cursor::{Cursor, PCursor, RCursor},
	Color32,
};
use std::fmt::{self, Debug};
use std::ops::BitXorAssign;

#[cfg(threading)]
use rayon::iter::ParallelIterator;

/// Represents the possible variations of Riemann Sums
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Riemann {
	Left,
	Middle,
	Right,
}

impl fmt::Display for Riemann {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

lazy_static::lazy_static! {
	/// Represents a "default" instance of `FunctionEntry`
	pub static ref DEFAULT_FUNCTION_ENTRY: FunctionEntry = FunctionEntry::default();
}

/// `FunctionEntry` is a function that can calculate values, integrals, derivatives, etc etc
#[derive(Clone)]
pub struct FunctionEntry {
	/// The `BackingFunction` instance that is used to generate `f(x)`, `f'(x)`, and `f''(x)`
	function: BackingFunction,

	/// Stores a function string (that hasn't been processed via `process_func_str`) to display to the user
	raw_func_str: String,

	/// Minimum and Maximum values of what do display
	min_x: f64,
	max_x: f64,

	/// If calculating/displayingintegrals are enabled
	pub integral: bool,

	/// If displaying derivatives are enabled (note, they are still calculated for other purposes)
	pub derivative: bool,

	pub nth_derviative: bool,

	back_data: Vec<Value>,
	integral_data: Option<(Vec<Bar>, f64)>,
	derivative_data: Vec<Value>,
	extrema_data: Vec<Value>,
	root_data: Vec<Value>,
	nth_derivative_data: Option<Vec<Value>>,

	autocomplete: AutoComplete<'static>,

	test_result: Option<String>,
	curr_nth: usize,

	pub settings_opened: bool,
}

impl Default for FunctionEntry {
	/// Creates default FunctionEntry instance (which is empty)
	fn default() -> FunctionEntry {
		FunctionEntry {
			function: BackingFunction::new("").unwrap(),
			raw_func_str: String::new(),
			min_x: -1.0,
			max_x: 1.0,
			integral: false,
			derivative: false,
			nth_derviative: false,
			back_data: Vec::new(),
			integral_data: None,
			derivative_data: Vec::new(),
			extrema_data: Vec::new(),
			root_data: Vec::new(),
			nth_derivative_data: None,
			autocomplete: AutoComplete::default(),
			test_result: None,
			curr_nth: 3,
			settings_opened: false,
		}
	}
}

impl FunctionEntry {
	pub fn function_entry(
		&mut self, ui: &mut egui::Ui, remove_i: &mut Option<usize>, can_remove: bool, i: usize,
	) {
		let output_string = self.autocomplete.string.clone();
		self.update_string(&output_string);

		let mut movement: Movement = Movement::default();

		let mut new_string = self.autocomplete.string.clone();

		let te_id = ui.make_persistent_id(format!("text_edit_ac_{}", i));
		let row_height = ui
			.fonts()
			.row_height(&egui::FontSelection::default().resolve(ui.style()));

		let max_size = vec2(ui.available_width(), {
			let had_focus = ui.ctx().memory().has_focus(te_id);
			let gotten_value = ui.ctx().animate_bool(te_id, had_focus);
			if gotten_value == 1.0 {
				row_height * 2.5
			} else {
				row_height * (1.0 + (gotten_value * 1.5))
			}
		});

		let re = ui.add_sized(
			max_size,
			egui::TextEdit::singleline(&mut new_string)
				.hint_forward(true) // Make the hint appear after the last text in the textbox
				.lock_focus(true)
				.id(te_id)
				.hint_text({
					if let Hint::Single(single_hint) = self.autocomplete.hint {
						*single_hint
					} else {
						""
					}
				}),
		);

		if ui.ctx().animate_bool(te_id, re.has_focus()) < 1.0 {
			return;
		}

		self.autocomplete.update_string(&new_string);

		if !self.autocomplete.hint.is_none() {
			if !self.autocomplete.hint.is_single() {
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

			self.autocomplete.register_movement(&movement);

			if movement != Movement::Complete && let Hint::Many(hints) = self.autocomplete.hint {
				// Doesn't need to have a number in id as there should only be 1 autocomplete popup in the entire gui
				let popup_id = ui.make_persistent_id("autocomplete_popup");

				let mut clicked = false;

				egui::popup_below_widget(ui, popup_id, &re, |ui| {
					hints.iter().enumerate().for_each(|(i, candidate)| {
						if ui.selectable_label(i == self.autocomplete.i, *candidate).clicked() {
							clicked = true;
							self.autocomplete.i = i;
						}
					});
				});

				if clicked {
					self.autocomplete.apply_hint(hints[self.autocomplete.i]);

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

		let buttons_area = egui::Area::new(format!("buttons_area_{}", i))
			.fixed_pos(pos2(re.rect.min.x, re.rect.min.y + (row_height * 1.32)))
			.order(egui::Order::Foreground);

		buttons_area.show(ui.ctx(), |ui| {
			ui.horizontal(|ui| {
				// There's more than 1 function! Functions can now be deleted
				if ui
					.add_enabled(can_remove, Button::new("✖").frame(false))
					.on_hover_text("Delete Function")
					.clicked()
				{
					*remove_i = Some(i);
				}

				// Toggle integral being enabled or not
				self.integral.bitxor_assign(
					ui.add(Button::new("∫").frame(false))
						.on_hover_text(match self.integral {
							true => "Don't integrate",
							false => "Integrate",
						})
						.clicked(),
				);

				// Toggle showing the derivative (even though it's already calculated this option just toggles if it's displayed or not)
				self.derivative.bitxor_assign(
					ui.add(Button::new("d/dx").frame(false))
						.on_hover_text(match self.derivative {
							true => "Don't Differentiate",
							false => "Differentiate",
						})
						.clicked(),
				);

				self.settings_opened.bitxor_assign(
					ui.add(Button::new("⚙").frame(false))
						.on_hover_text(match self.settings_opened {
							true => "Close Settings",
							false => "Open Settings",
						})
						.clicked(),
				);
			});
		});
	}

	pub fn settings_window(&mut self, ctx: &Context) {
		let mut invalidate_nth = false;
		egui::Window::new(format!("Settings: {}", self.raw_func_str))
			.open(&mut self.settings_opened)
			.default_pos([200.0, 200.0])
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.add(Checkbox::new(
					&mut self.nth_derviative,
					"Display Nth Derivative",
				));

				if ui
					.add(egui::Slider::new(&mut self.curr_nth, 3..=5).text("Nth Derivative"))
					.changed()
				{
					invalidate_nth = true;
				}
			});

		if invalidate_nth {
			self.invalidate_nth();
		}
	}

	/// Get function's cached test result
	pub fn get_test_result(&self) -> &Option<String> { &self.test_result }

	/// Update function string and test it
	fn update_string(&mut self, raw_func_str: &str) {
		if raw_func_str == self.raw_func_str {
			return;
		}

		self.raw_func_str = raw_func_str.to_string();
		let processed_func = process_func_str(raw_func_str);
		let new_func_result = BackingFunction::new(&processed_func);

		match new_func_result {
			Ok(new_function) => {
				self.test_result = None;
				self.function = new_function;
				self.invalidate_whole();
			}
			Err(error) => {
				self.test_result = Some(error);
			}
		}
	}

	/// Get function that can be used to calculate integral based on Riemann Sum type
	fn get_sum_func(&self, sum: Riemann) -> FunctionHelper {
		match sum {
			Riemann::Left => {
				FunctionHelper::new(|left_x: f64, _: f64| -> f64 { self.function.get(left_x) })
			}
			Riemann::Right => {
				FunctionHelper::new(|_: f64, right_x: f64| -> f64 { self.function.get(right_x) })
			}
			Riemann::Middle => FunctionHelper::new(|left_x: f64, right_x: f64| -> f64 {
				(self.function.get(left_x) + self.function.get(right_x)) / 2.0
			}),
		}
	}

	/// Creates and does the math for creating all the rectangles under the graph
	fn integral_rectangles(
		&self, integral_min_x: &f64, integral_max_x: &f64, sum: &Riemann, integral_num: &usize,
	) -> (Vec<(f64, f64)>, f64) {
		if integral_min_x.is_nan() {
			panic!("integral_min_x is NaN")
		} else if integral_max_x.is_nan() {
			panic!("integral_max_x is NaN")
		}

		let step = (integral_min_x - integral_max_x).abs() / (*integral_num as f64);

		let sum_func = self.get_sum_func(*sum);

		let data2: Vec<(f64, f64)> = dyn_iter(&step_helper(*integral_num, integral_min_x, &step))
			.map(|x| {
				let step_offset = step * x.signum(); // store the offset here so it doesn't have to be calculated multiple times
				let x2: f64 = x + step_offset;

				let (left_x, right_x) = match x.is_sign_positive() {
					true => (*x, x2),
					false => (x2, *x),
				};

				let y = sum_func.get(left_x, right_x);

				(x + (step_offset / 2.0), y)
			})
			.filter(|(_, y)| !y.is_nan())
			.collect();

		let area = data2.iter().map(|(_, y)| y * step).sum();

		(data2, area)
	}

	/// Helps with processing newton's method depending on level of derivative
	fn newtons_method_helper(&self, threshold: &f64, derivative_level: usize) -> Vec<Value> {
		let range = self.min_x..self.max_x;
		let newtons_method_output: Vec<f64> = match derivative_level {
			0 => newtons_method_helper(
				threshold,
				&range,
				self.back_data.as_slice(),
				&|x: f64| self.function.get(x),
				&|x: f64| self.function.get_derivative_1(x),
			),
			1 => newtons_method_helper(
				threshold,
				&range,
				self.derivative_data.as_slice(),
				&|x: f64| self.function.get_derivative_1(x),
				&|x: f64| self.function.get_derivative_2(x),
			),
			_ => unreachable!(),
		};

		dyn_iter(&newtons_method_output)
			.map(|x| Value::new(*x, self.function.get(*x)))
			.collect()
	}

	/// Does the calculations and stores results in `self`
	pub fn calculate(
		&mut self, min_x: &f64, max_x: &f64, width_changed: bool, settings: &AppSettings,
	) {
		if self.test_result.is_some() {
			return;
		}

		let resolution: f64 = settings.plot_width as f64 / (max_x.abs() + min_x.abs());
		let resolution_iter = resolution_helper(&settings.plot_width + 1, min_x, &resolution);

		// Makes sure proper arguments are passed when integral is enabled
		if self.integral && settings.integral_changed {
			self.invalidate_integral();
		}

		let mut partial_regen = false;
		let min_max_changed = (min_x != &self.min_x) | (max_x != &self.max_x);

		let derivative_required = settings.do_extrema | self.derivative;

		self.min_x = *min_x;
		self.max_x = *max_x;
		if width_changed {
			self.invalidate_back();
			self.invalidate_derivative();
		} else if min_max_changed && !self.back_data.is_empty() {
			partial_regen = true;

			let x_data: SteppedVector = self
				.back_data
				.iter()
				.map(|ele| ele.x)
				.collect::<Vec<f64>>()
				.into();

			let back_data: Vec<Value> = dyn_iter(&resolution_iter)
				.map(|x| {
					if let Some(i) = x_data.get_index(x) {
						self.back_data[i]
					} else {
						Value::new(*x, self.function.get(*x))
					}
				})
				.collect();

			debug_assert_eq!(back_data.len(), settings.plot_width + 1);

			self.back_data = back_data;

			if derivative_required {
				let new_derivative_data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| {
						if let Some(i) = x_data.get_index(x) {
							self.derivative_data[i]
						} else {
							Value::new(*x, self.function.get_derivative_1(*x))
						}
					})
					.collect();

				debug_assert_eq!(new_derivative_data.len(), settings.plot_width + 1);

				self.derivative_data = new_derivative_data;
			} else {
				self.invalidate_derivative();
			}

			if self.nth_derviative && let Some(nth_derivative_data) = &self.nth_derivative_data {
				let new_nth_derivative_data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| {
						if let Some(i) = x_data.get_index(x) {
							(*nth_derivative_data)[i]
						} else {
							Value::new(*x, self.function.get_nth_derivative(self.curr_nth, *x))
						}
					})
					.collect();

				debug_assert_eq!(new_nth_derivative_data.len(), settings.plot_width + 1);

				self.nth_derivative_data = Some(new_nth_derivative_data);
			} else {
				self.invalidate_nth();
			}
		} else {
			self.invalidate_back();
			self.invalidate_derivative();
		}

		let threshold: f64 = resolution / 2.0;

		if !partial_regen {
			if self.back_data.is_empty() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get(*x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);

				self.back_data = data;
			}

			if derivative_required && self.derivative_data.is_empty() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get_derivative_1(*x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);
				self.derivative_data = data;
			}

			if self.nth_derviative && self.nth_derivative_data.is_none() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get_nth_derivative(self.curr_nth, *x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);
				self.nth_derivative_data = Some(data);
			}
		}

		if self.integral {
			if self.integral_data.is_none() {
				let (data, area) = self.integral_rectangles(
					&settings.integral_min_x,
					&settings.integral_max_x,
					&settings.riemann_sum,
					&settings.integral_num,
				);
				self.integral_data =
					Some((data.iter().map(|(x, y)| Bar::new(*x, *y)).collect(), area));
			}
		} else {
			self.invalidate_integral();
		}

		// Calculates extrema
		if settings.do_extrema && (min_max_changed | self.extrema_data.is_empty()) {
			self.extrema_data = self.newtons_method_helper(&threshold, 1);
		}

		// Calculates roots
		if settings.do_roots && (min_max_changed | self.root_data.is_empty()) {
			self.root_data = self.newtons_method_helper(&threshold, 0);
		}
	}

	/// Displays the function's output on PlotUI `plot_ui` with settings `settings`.
	/// Returns an `Option<f64>` of the calculated integral.
	pub fn display(
		&self, plot_ui: &mut PlotUi, settings: &AppSettings, main_plot_color: Color32,
	) -> Option<f64> {
		if self.test_result.is_some() {
			return None;
		}

		let derivative_str = self.function.get_derivative_str();
		let step = (settings.integral_min_x - settings.integral_max_x).abs()
			/ (settings.integral_num as f64);

		let resolution = (self.min_x - self.max_x).abs() / (settings.plot_width as f64);

		// Plot back data
		if !self.back_data.is_empty() {
			if self.integral && (resolution >= step) {
				plot_ui.line(
					self.back_data
						.iter()
						.filter(|value| {
							(value.x > settings.integral_min_x)
								&& (settings.integral_max_x > value.x)
						})
						.cloned()
						.collect::<Vec<Value>>()
						.to_line()
						.color(Color32::BLUE)
						.name(&self.raw_func_str)
						.fill(0.0),
				);
			}
			plot_ui.line(
				self.back_data
					.to_line()
					.color(main_plot_color)
					.name(&self.raw_func_str),
			);
		}

		// Plot derivative data
		if self.derivative && !self.derivative_data.is_empty() {
			plot_ui.line(
				self.derivative_data
					.to_line()
					.color(Color32::GREEN)
					.name(derivative_str),
			);
		}

		// Plot extrema points
		if settings.do_extrema && !self.extrema_data.is_empty() {
			plot_ui.points(
				self.extrema_data
					.to_points()
					.color(Color32::YELLOW)
					.name("Extrema")
					.radius(5.0), // Radius of points of Extrema
			);
		}

		// Plot roots points
		if settings.do_roots && !self.root_data.is_empty() {
			plot_ui.points(
				self.root_data
					.to_points()
					.color(Color32::LIGHT_BLUE)
					.name("Root")
					.radius(5.0), // Radius of points of Roots
			);
		}

		if self.nth_derviative && let Some(nth_derviative) = &self.nth_derivative_data {
			plot_ui.line(
				(*nth_derviative)
					.to_line()
					.color(Color32::DARK_RED)
					.name(self.function.get_nth_derivative_str()),
			);
		}

		// Plot integral data
		match &self.integral_data {
			Some(integral_data) => {
				if step > resolution {
					plot_ui.bar_chart(
						BarChart::new(integral_data.0.clone())
							.color(Color32::BLUE)
							.width(step),
					);
				}

				// return value rounded to 8 decimal places
				Some(crate::misc::decimal_round(integral_data.1, 8))
			}
			_ => None,
		}
	}

	/// Invalidate entire cache
	pub fn invalidate_whole(&mut self) {
		self.invalidate_back();
		self.invalidate_integral();
		self.invalidate_derivative();
		self.invalidate_nth();
		self.extrema_data.clear();
		self.root_data.clear();
	}

	/// Invalidate `back` data
	pub fn invalidate_back(&mut self) { self.back_data.clear(); }

	/// Invalidate Integral data
	pub fn invalidate_integral(&mut self) { self.integral_data = None; }

	/// Invalidate Derivative data
	pub fn invalidate_derivative(&mut self) { self.derivative_data.clear(); }

	pub fn invalidate_nth(&mut self) { self.nth_derivative_data = None }

	/// Runs asserts to make sure everything is the expected value
	#[cfg(test)]
	pub fn tests(
		&mut self, settings: AppSettings, back_target: Vec<(f64, f64)>,
		derivative_target: Vec<(f64, f64)>, area_target: f64, min_x: f64, max_x: f64,
	) {
		{
			self.calculate(&min_x, &max_x, true, &settings);
			let back_target = back_target;
			assert!(!self.back_data.is_empty());
			assert_eq!(self.back_data.len(), settings.plot_width + 1);
			let back_vec_tuple = self.back_data.to_tuple();
			assert_eq!(back_vec_tuple, back_target);

			assert!(self.integral);
			assert!(self.derivative);

			assert_eq!(!self.root_data.is_empty(), settings.do_roots);
			assert_eq!(!self.extrema_data.is_empty(), settings.do_extrema);
			assert!(!self.derivative_data.is_empty());
			assert!(self.integral_data.is_some());

			assert_eq!(self.derivative_data.to_tuple(), derivative_target);

			assert_eq!(self.integral_data.clone().unwrap().1, area_target);
		}

		{
			self.update_string("sin(x)");
			assert!(self.get_test_result().is_none());
			assert_eq!(&self.raw_func_str, "sin(x)");

			self.integral = false;
			self.derivative = false;

			assert!(!self.integral);
			assert!(!self.derivative);

			assert!(self.back_data.is_empty());
			assert!(self.integral_data.is_none());
			assert!(self.root_data.is_empty());
			assert!(self.extrema_data.is_empty());
			assert!(self.derivative_data.is_empty());

			self.calculate(&min_x, &max_x, true, &settings);

			assert!(!self.back_data.is_empty());
			assert!(self.integral_data.is_none());
			assert!(self.root_data.is_empty());
			assert!(self.extrema_data.is_empty());
			assert!(self.derivative_data.is_empty());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn app_settings_constructor(
		sum: Riemann, integral_min_x: f64, integral_max_x: f64, pixel_width: usize,
		integral_num: usize,
	) -> AppSettings {
		crate::math_app::AppSettings {
			riemann_sum: sum,
			integral_min_x,
			integral_max_x,
			integral_changed: true,
			integral_num,
			do_extrema: false,
			do_roots: false,
			plot_width: pixel_width,
		}
	}

	static BACK_TARGET: [(f64, f64); 11] = [
		(-1.0, 1.0),
		(-0.8, 0.6400000000000001),
		(-0.6, 0.36),
		(-0.4, 0.16000000000000003),
		(-0.19999999999999996, 0.03999999999999998),
		(0.0, 0.0),
		(0.19999999999999996, 0.03999999999999998),
		(0.3999999999999999, 0.15999999999999992),
		(0.6000000000000001, 0.3600000000000001),
		(0.8, 0.6400000000000001),
		(1.0, 1.0),
	];

	static DERIVATIVE_TARGET: [(f64, f64); 11] = [
		(-1.0, -2.0),
		(-0.8, -1.6),
		(-0.6, -1.2),
		(-0.4, -0.8),
		(-0.19999999999999996, -0.3999999999999999),
		(0.0, 0.0),
		(0.19999999999999996, 0.3999999999999999),
		(0.3999999999999999, 0.7999999999999998),
		(0.6000000000000001, 1.2000000000000002),
		(0.8, 1.6),
		(1.0, 2.0),
	];

	fn do_test(sum: Riemann, area_target: f64) {
		let settings = app_settings_constructor(sum, -1.0, 1.0, 10, 10);

		let mut function = FunctionEntry::default();
		function.update_string("x^2");
		function.integral = true;
		function.derivative = true;

		function.tests(
			settings,
			BACK_TARGET.to_vec(),
			DERIVATIVE_TARGET.to_vec(),
			area_target,
			-1.0,
			1.0,
		);
	}

	#[test]
	fn function_entry_test() {
		do_test(Riemann::Left, 0.9600000000000001);
		do_test(Riemann::Middle, 0.92);
		do_test(Riemann::Right, 0.8800000000000001);
	}
}
