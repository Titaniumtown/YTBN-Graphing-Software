#![allow(clippy::too_many_arguments)] // Clippy, shut

use crate::math_app::AppSettings;
use crate::misc::*;
use egui::{
	plot::{BarChart, PlotUi, Value},
	widgets::plot::Bar,
	Checkbox, Context,
};
use epaint::Color32;
use parsing::{generate_hint, AutoComplete};
use parsing::{process_func_str, BackingFunction};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::{
	fmt::{self, Debug},
	intrinsics::assume,
};

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

/// `FunctionEntry` is a function that can calculate values, integrals, derivatives, etc etc
#[derive(PartialEq, Clone)]
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

	pub autocomplete: AutoComplete<'static>,

	test_result: Option<String>,
	curr_nth: usize,

	pub settings_opened: bool,
}

impl Serialize for FunctionEntry {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut s = serializer.serialize_struct("FunctionEntry", 4)?;
		s.serialize_field("raw_func_str", &self.raw_func_str)?;
		s.serialize_field("integral", &self.integral)?;
		s.serialize_field("derivative", &self.derivative)?;
		s.serialize_field("curr_nth", &self.curr_nth)?;

		s.end()
	}
}

impl<'de> Deserialize<'de> for FunctionEntry {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Helper {
			raw_func_str: String,
			integral: bool,
			derivative: bool,
			curr_nth: usize,
		}

		let helper = Helper::deserialize(deserializer)?;
		let mut new_func_entry = FunctionEntry::EMPTY;
		let gen_func = BackingFunction::new(&helper.raw_func_str);
		match gen_func {
			Ok(func) => new_func_entry.function = func,
			Err(x) => new_func_entry.test_result = Some(x),
		}

		new_func_entry.autocomplete = AutoComplete {
			i: 0,
			hint: generate_hint(&helper.raw_func_str),
			string: helper.raw_func_str,
		};

		new_func_entry.integral = helper.integral;
		new_func_entry.derivative = helper.derivative;
		new_func_entry.curr_nth = helper.curr_nth;

		Ok(new_func_entry)
	}
}

impl const Default for FunctionEntry {
	/// Creates default FunctionEntry instance (which is empty)
	fn default() -> FunctionEntry { FunctionEntry::EMPTY }
}

impl FunctionEntry {
	pub const EMPTY: FunctionEntry = FunctionEntry {
		function: BackingFunction::EMPTY,
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
	};

	pub const fn is_some(&self) -> bool { !self.function.is_none() }

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
	pub fn update_string(&mut self, raw_func_str: &str) {
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

		unsafe { assume(!resolution_iter.is_empty()) }

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
				.as_slice()
				.into();

			let (back_data, derivative_data_1): (Vec<Value>, Vec<Option<Value>>) =
				dyn_iter(&resolution_iter)
					.map(|x| {
						if let Some(i) = x_data.get_index(x) {
							(
								self.back_data[i],
								derivative_required.then(|| self.derivative_data[i]),
							)
						} else {
							(
								Value::new(*x, self.function.get(*x)),
								derivative_required
									.then(|| Value::new(*x, self.function.get_derivative_1(*x))),
							)
						}
					})
					.collect::<Vec<(Value, Option<Value>)>>()
					.into_iter()
					.unzip();

			debug_assert_eq!(back_data.len(), settings.plot_width + 1);
			debug_assert_eq!(derivative_data_1.len(), settings.plot_width + 1);

			self.back_data = back_data;

			if derivative_required {
				debug_assert!(derivative_data_1[0].is_some());
				self.derivative_data = derivative_data_1
					.into_iter()
					.map(|ele| unsafe { ele.unwrap_unchecked() })
					.collect::<Vec<Value>>();
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

				unsafe {
					assume(!self.derivative_data.is_empty());
				}
			}

			if self.nth_derviative && self.nth_derivative_data.is_none() {
				let data: Vec<Value> = dyn_iter(&resolution_iter)
					.map(|x| Value::new(*x, self.function.get_nth_derivative(self.curr_nth, *x)))
					.collect();
				debug_assert_eq!(data.len(), settings.plot_width + 1);
				self.nth_derivative_data = Some(data);

				unsafe {
					assume(self.nth_derivative_data.is_some());
				}
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
				unsafe {
					assume(self.integral_data.is_some());
				}
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
		if self.test_result.is_some() | self.function.is_none() {
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
					.stroke(egui::Stroke::new(2.0, main_plot_color))
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
	#[allow(dead_code)]
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
