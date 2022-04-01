#![allow(clippy::too_many_arguments)] // Clippy, shut

use crate::math_app::AppSettings;
use crate::misc::*;
use crate::parsing::{process_func_str, BackingFunction};
use crate::suggestions::{generate_hint, HintEnum};
use eframe::{egui, epaint};
use egui::{
	epaint::text::cursor::Cursor, text::CCursor, text_edit::CursorRange, Key, TextEdit, Widget,
};
use egui::{
	plot::{BarChart, PlotUi, Value},
	widgets::plot::Bar,
};
use epaint::text::cursor::{PCursor, RCursor};
use epaint::Color32;
use std::fmt::{self, Debug};

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

#[derive(Clone)]
struct AutoComplete {
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
	fn changed(&mut self, string: String) {
		if self.func_str != Some(string.clone()) {
			self.changed = true;
			self.func_str = Some(string.clone());
			self.hint = generate_hint(string);
		} else {
			self.changed = false;
		}
	}
}

/// `FunctionEntry` is a function that can calculate values, integrals,
/// derivatives, etc etc
#[derive(Clone)]
pub struct FunctionEntry {
	/// The `BackingFunction` instance that is used to generate `f(x)`, `f'(x)`,
	/// and `f''(x)`
	function: BackingFunction,

	/// Stores a function string (that hasn't been processed via
	/// `process_func_str`) to display to the user
	raw_func_str: String,

	/// Minimum and Maximum values of what do display
	min_x: f64,
	max_x: f64,

	/// If calculating/displayingintegrals are enabled
	pub integral: bool,

	/// If displaying derivatives are enabled (note, they are still calculated
	/// for other purposes)
	pub derivative: bool,

	back_data: Option<Vec<Value>>,
	integral_data: Option<(Vec<Bar>, f64)>,
	derivative_data: Option<Vec<Value>>,
	extrema_data: Option<Vec<Value>>,
	roots_data: Option<Vec<Value>>,

	autocomplete: AutoComplete,

	test_result: Option<String>,
}

impl Default for FunctionEntry {
	/// Creates default FunctionEntry instance (which is empty)
	fn default() -> FunctionEntry {
		FunctionEntry {
			function: BackingFunction::new(""),
			raw_func_str: String::new(),
			min_x: -1.0,
			max_x: 1.0,
			integral: false,
			derivative: false,
			back_data: None,
			integral_data: None,
			derivative_data: None,
			extrema_data: None,
			roots_data: None,
			autocomplete: AutoComplete::default(),
			test_result: None,
		}
	}
}

impl FunctionEntry {
	pub fn get_func_raw(&self) -> String { self.raw_func_str.to_string() }

	pub fn auto_complete(
		&mut self, ui: &mut egui::Ui, string: &mut String,
	) -> (bool, bool, Option<String>) {
		// Put here so these key presses don't interact with other elements
		let enter_pressed = ui
			.input_mut()
			.consume_key(egui::Modifiers::NONE, Key::Enter);
		let tab_pressed = ui.input_mut().consume_key(egui::Modifiers::NONE, Key::Tab);

		let te_id = ui.make_persistent_id("text_edit_ac".to_string());

		// update self.autocomplete
		self.autocomplete.changed(string.clone());

		let mut func_edit = egui::TextEdit::singleline(string)
			.hint_forward(true)
			.lock_focus(true);

		if self.autocomplete.hint.is_none() {
			func_edit.id(te_id).ui(ui);
			return (false, false, self.get_test_result());
		}

		if let Some(single_hint) = self.autocomplete.hint.get_single() {
			let func_edit_2 = func_edit;
			func_edit = func_edit_2.hint_text(&single_hint);
		}

		let re = func_edit.id(te_id).ui(ui);

		let func_edit_focus = re.has_focus();

		// If in focus and right arrow key was pressed, apply hint
		if func_edit_focus {
			let mut push_cursor: bool = false;
			let apply_key = ui.input().key_pressed(Key::ArrowRight) | enter_pressed | tab_pressed;

			if apply_key && let Some(single_hint) = self.autocomplete.hint.get_single() {
					push_cursor = true;
					*string = string.clone() + &single_hint;
			} else if self.autocomplete.hint.is_multi() {
				let selections = self.autocomplete.hint.ensure_many();

				let max_i = selections.len() as i16 - 1;

				let mut i = self.autocomplete.i as i16;

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

				self.autocomplete.i = i as usize;

				let popup_id = ui.make_persistent_id("autocomplete_popup");

				let mut clicked = false;

				egui::popup_below_widget(ui, popup_id, &re, |ui| {
					for (i, candidate) in selections.iter().enumerate() {
						if ui
							.selectable_label(i == self.autocomplete.i, *candidate)
							.clicked()
						{
							clicked = true;
							self.autocomplete.i = i;
						}
					}
				});

				if clicked | apply_key {
					*string += selections[self.autocomplete.i];
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

		let changed = *string != self.get_func_raw();
		if changed {
			self.update_string(&*string);
		}

		(func_edit_focus, changed, self.get_test_result())
	}

	pub fn get_test_result(&self) -> Option<String> { self.test_result.clone() }

	fn update_string(&mut self, raw_func_str: &str) {
		let processed_func = process_func_str(raw_func_str);
		let output = crate::parsing::test_func(&processed_func);
		self.raw_func_str = raw_func_str.to_string();
		if output.is_some() {
			self.test_result = output;
			return;
		} else {
			self.test_result = None;
		}

		self.function = BackingFunction::new(&processed_func);
		self.invalidate_whole();
	}

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

	/// Creates and does the math for creating all the rectangles under the
	/// graph
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
	fn newtons_method_helper(
		&self, threshold: &f64, derivative_level: usize,
	) -> Option<Vec<Value>> {
		let range = self.min_x..self.max_x;
		let newtons_method_output: Vec<f64> = match derivative_level {
			0 => newtons_method_helper(
				threshold,
				&range,
				self.back_data.as_ref().unwrap(),
				&|x: f64| self.function.get(x),
				&|x: f64| self.function.get_derivative_1(x),
			),
			1 => newtons_method_helper(
				threshold,
				&range,
				self.derivative_data.as_ref().unwrap(),
				&|x: f64| self.function.get_derivative_1(x),
				&|x: f64| self.function.get_derivative_2(x),
			),
			_ => unreachable!(),
		};

		if newtons_method_output.is_empty() {
			None
		} else {
			Some(
				dyn_iter(&newtons_method_output)
					.map(|x| Value::new(*x, self.function.get(*x)))
					.collect(),
			)
		}
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
		} else if min_max_changed && self.back_data.is_some() {
			partial_regen = true;

			let back_cache = self.back_data.as_ref().unwrap();

			let x_data: SteppedVector = back_cache
				.iter()
				.map(|ele| ele.x)
				.collect::<Vec<f64>>()
				.into();

			let back_data: Vec<Value> = dyn_iter(&resolution_iter)
				.map(|x| {
					if let Some(i) = x_data.get_index(x) {
						back_cache[i]
					} else {
						Value::new(*x, self.function.get(*x))
					}
				})
				.collect();
			debug_assert_eq!(back_data.len(), settings.plot_width + 1);
			self.back_data = Some(back_data);

			if derivative_required {
				let derivative_cache = self.derivative_data.as_ref().unwrap();
				let new_derivative_data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| {
						if let Some(i) = x_data.get_index(x) {
							derivative_cache[i]
						} else {
							Value::new(*x, self.function.get_derivative_1(*x))
						}
					})
					.collect();

				debug_assert_eq!(new_derivative_data.len(), settings.plot_width + 1);

				self.derivative_data = Some(new_derivative_data);
			} else {
				self.derivative_data = None;
			}
		} else {
			self.invalidate_back();
			self.invalidate_derivative();
		}

		let threshold: f64 = resolution / 2.0;

		if !partial_regen {
			if self.back_data.is_none() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get(*x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);

				self.back_data = Some(data);
			}

			if derivative_required && self.derivative_data.is_none() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get_derivative_1(*x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);
				self.derivative_data = Some(data);
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
		if settings.do_extrema && (min_max_changed | self.extrema_data.is_none()) {
			self.extrema_data = self.newtons_method_helper(&threshold, 1);
		}

		// Calculates roots
		if settings.do_roots && (min_max_changed | self.roots_data.is_none()) {
			self.roots_data = self.newtons_method_helper(&threshold, 0);
		}
	}

	/// Displays the function's output on PlotUI `plot_ui` with settings
	/// `settings`. Returns an `Option<f64>` of the calculated integral
	pub fn display(&self, plot_ui: &mut PlotUi, settings: &AppSettings) -> Option<f64> {
		if self.test_result.is_some() {
			return None;
		}

		let derivative_str = self.function.get_derivative_str();
		let step = (settings.integral_min_x - settings.integral_max_x).abs()
			/ (settings.integral_num as f64);
		// Plot back data
		if let Some(back_data) = &self.back_data {
			plot_ui.line(
				back_data
					.clone()
					.to_line()
					.color(Color32::RED)
					.name(self.get_func_raw()),
			);
		}

		// Plot derivative data
		if self.derivative {
			if let Some(derivative_data) = &self.derivative_data {
				plot_ui.line(
					derivative_data
						.clone()
						.to_line()
						.color(Color32::GREEN)
						.name(derivative_str),
				);
			}
		}

		// Plot extrema points
		if settings.do_extrema {
			if let Some(extrema_data) = &self.extrema_data {
				plot_ui.points(
					extrema_data
						.clone()
						.to_points()
						.color(Color32::YELLOW)
						.name("Extrema")
						.radius(5.0), // Radius of points of Extrema
				);
			}
		}

		// Plot roots points
		if settings.do_roots {
			if let Some(roots_data) = &self.roots_data {
				plot_ui.points(
					roots_data
						.clone()
						.to_points()
						.color(Color32::LIGHT_BLUE)
						.name("Root")
						.radius(5.0), // Radius of points of Roots
				);
			}
		}

		// Plot integral data
		if let Some(integral_data) = &self.integral_data {
			plot_ui.bar_chart(
				BarChart::new(integral_data.0.clone())
					.color(Color32::BLUE)
					.width(step),
			);

			// return value rounded to 8 decimal places
			Some(crate::misc::decimal_round(integral_data.1, 8))
		} else {
			None
		}
	}

	/// Invalidate entire cache
	pub fn invalidate_whole(&mut self) {
		self.back_data = None;
		self.integral_data = None;
		self.derivative_data = None;
		self.extrema_data = None;
		self.roots_data = None;
	}

	/// Invalidate `back` data
	pub fn invalidate_back(&mut self) { self.back_data = None; }

	/// Invalidate Integral data
	pub fn invalidate_integral(&mut self) { self.integral_data = None; }

	/// Invalidate Derivative data
	pub fn invalidate_derivative(&mut self) { self.derivative_data = None; }

	/// Depreciated, but still used for tests
	#[cfg(test)]
	pub fn update(
		&mut self, raw_func_str: &str, integral: bool, derivative: bool,
	) -> Option<String> {
		self.derivative = derivative;
		self.integral = integral;
		if raw_func_str != self.get_func_raw() {
			self.update_string(raw_func_str);
			self.get_test_result()
		} else {
			None
		}
	}

	/// Runs asserts to make sure everything is the expected value
	#[cfg(test)]
	pub fn tests(
		&mut self, settings: AppSettings, back_target: Vec<(f64, f64)>,
		derivative_target: Vec<(f64, f64)>, area_target: f64, min_x: f64, max_x: f64,
	) {
		{
			self.calculate(&min_x, &max_x, true, &settings);
			let settings = settings;
			let back_target = back_target;
			assert!(self.back_data.is_some());
			let back_data = self.back_data.as_ref().unwrap().clone();
			assert_eq!(back_data.len(), settings.plot_width + 1);
			let back_vec_tuple = back_data.to_tuple();
			assert_eq!(back_vec_tuple, back_target);

			assert!(self.integral);
			assert!(self.derivative);

			assert_eq!(self.roots_data.is_some(), settings.do_roots);
			assert_eq!(self.extrema_data.is_some(), settings.do_extrema);
			assert!(self.derivative_data.is_some());
			assert!(self.integral_data.is_some());

			assert_eq!(
				self.derivative_data.as_ref().unwrap().to_tuple(),
				derivative_target
			);

			assert_eq!(self.integral_data.clone().unwrap().1, area_target);
		}

		{
			self.update("x^3", false, false);
			assert!(!self.integral);
			assert!(!self.derivative);

			assert!(self.back_data.is_none());
			assert!(self.integral_data.is_none());
			assert!(self.roots_data.is_none());
			assert!(self.extrema_data.is_none());
			assert!(self.derivative_data.is_none());

			self.calculate(&min_x, &max_x, true, &settings);

			assert!(self.back_data.is_some());
			assert!(self.integral_data.is_none());
			assert!(self.roots_data.is_none());
			assert!(self.extrema_data.is_none());
			assert!(self.derivative_data.is_none());
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
		function.update("x^2", true, true);

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
