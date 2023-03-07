use crate::math_app::AppSettings;
use crate::misc::*;
use egui::{
	plot::{BarChart, PlotPoint, PlotUi},
	widgets::plot::Bar,
	Checkbox, Context,
};
use epaint::Color32;
use parsing::{generate_hint, AutoComplete};
use parsing::{process_func_str, BackingFunction};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::{
	fmt::{self, Debug},
	hash::{Hash, Hasher},
	intrinsics::assume,
};

/// Represents the possible variations of Riemann Sums
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Riemann {
	Left,
	Middle,
	Right,
}

impl fmt::Display for Riemann {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

impl const Default for Riemann {
	fn default() -> Riemann { Riemann::Left }
}

/// `FunctionEntry` is a function that can calculate values, integrals, derivatives, etc etc
#[derive(Clone)]
pub struct FunctionEntry {
	/// The `BackingFunction` instance that is used to generate `f(x)`, `f'(x)`, and `f''(x)`
	function: BackingFunction,

	/// Stores a function string (that hasn't been processed via `process_func_str`) to display to the user
	pub raw_func_str: String,

	/// If calculating/displayingintegrals are enabled
	pub integral: bool,

	/// If displaying derivatives are enabled (note, they are still calculated for other purposes)
	pub derivative: bool,

	pub nth_derviative: bool,

	pub back_data: Vec<PlotPoint>,
	pub integral_data: Option<(Vec<Bar>, f64)>,
	pub derivative_data: Vec<PlotPoint>,
	pub extrema_data: Vec<PlotPoint>,
	pub root_data: Vec<PlotPoint>,
	nth_derivative_data: Option<Vec<PlotPoint>>,

	pub autocomplete: AutoComplete<'static>,

	test_result: Option<String>,
	curr_nth: usize,

	pub settings_opened: bool,
}

impl Hash for FunctionEntry {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.raw_func_str.hash(state);
		self.integral.hash(state);
		self.nth_derviative.hash(state);
		self.curr_nth.hash(state);
		self.settings_opened.hash(state);
	}
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
			self.clear_nth();
		}
	}

	/// Get function's cached test result
	pub fn get_test_result(&self) -> &Option<String> { &self.test_result }

	/// Update function string and test it
	pub fn update_string(&mut self, raw_func_str: &str) {
		if raw_func_str == self.raw_func_str {
			return;
		}

		self.raw_func_str = raw_func_str.to_owned();
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

	/// Creates and does the math for creating all the rectangles under the graph
	fn integral_rectangles(
		&self, integral_min_x: f64, integral_max_x: f64, sum: Riemann, integral_num: usize,
	) -> (Vec<(f64, f64)>, f64) {
		let step = (integral_max_x - integral_min_x) / (integral_num as f64);

		// let sum_func = self.get_sum_func(sum);

		let data2: Vec<(f64, f64)> = step_helper(integral_num, integral_min_x, step)
			.into_iter()
			.map(|x| {
				let step_offset = step.copysign(x); // store the offset here so it doesn't have to be calculated multiple times
				let x2: f64 = x + step_offset;

				let (left_x, right_x) = match x.is_sign_positive() {
					true => (x, x2),
					false => (x2, x),
				};

				let y = match sum {
					Riemann::Left => self.function.get(left_x),
					Riemann::Right => self.function.get(right_x),
					Riemann::Middle => {
						(self.function.get(left_x) + self.function.get(right_x)) / 2.0
					}
				};

				(x + (step_offset / 2.0), y)
			})
			.filter(|(_, y)| y.is_finite())
			.collect();

		let area = data2.iter().map(move |(_, y)| y * step).sum();

		(data2, area)
	}

	/// Helps with processing newton's method depending on level of derivative
	fn newtons_method_helper(
		&self, threshold: f64, derivative_level: usize, range: &std::ops::Range<f64>,
	) -> Vec<PlotPoint> {
		let newtons_method_output: Vec<f64> = match derivative_level {
			0 => newtons_method_helper(
				threshold,
				range,
				self.back_data.as_slice(),
				&|x: f64| self.function.get(x),
				&|x: f64| self.function.get_derivative_1(x),
			),
			1 => newtons_method_helper(
				threshold,
				range,
				self.derivative_data.as_slice(),
				&|x: f64| self.function.get_derivative_1(x),
				&|x: f64| self.function.get_derivative_2(x),
			),
			_ => unreachable!(),
		};

		newtons_method_output
			.into_iter()
			.map(|x| PlotPoint::new(x, self.function.get(x)))
			.collect()
	}

	/// Does the calculations and stores results in `self`
	pub fn calculate(
		&mut self, width_changed: bool, min_max_changed: bool, did_zoom: bool,
		settings: AppSettings,
	) {
		if self.test_result.is_some() | self.function.is_none() {
			return;
		}

		let resolution = (settings.max_x - settings.min_x) / (settings.plot_width as f64);
		debug_assert!(resolution > 0.0);
		let resolution_iter = step_helper(settings.plot_width + 1, settings.min_x, resolution);

		unsafe { assume(!resolution_iter.is_empty()) }

		// Makes sure proper arguments are passed when integral is enabled
		if self.integral && settings.integral_changed {
			self.clear_integral();
		}

		if width_changed | min_max_changed | did_zoom {
			self.clear_back();
			self.clear_derivative();
			self.clear_nth();
		}

		if self.back_data.is_empty() {
			let data: Vec<PlotPoint> = resolution_iter
				.clone()
				.into_iter()
				.map(|x| PlotPoint::new(x, self.function.get(x)))
				.collect();
			debug_assert_eq!(data.len(), settings.plot_width + 1);

			self.back_data = data;
		}

		if self.derivative_data.is_empty() {
			let data: Vec<PlotPoint> = resolution_iter
				.clone()
				.into_iter()
				.map(|x| PlotPoint::new(x, self.function.get_derivative_1(x)))
				.collect();
			debug_assert_eq!(data.len(), settings.plot_width + 1);
			self.derivative_data = data;
		}

		if self.nth_derviative && self.nth_derivative_data.is_none() {
			let data: Vec<PlotPoint> = resolution_iter
				.into_iter()
				.map(|x| PlotPoint::new(x, self.function.get_nth_derivative(self.curr_nth, x)))
				.collect();
			debug_assert_eq!(data.len(), settings.plot_width + 1);
			self.nth_derivative_data = Some(data);
		}

		if self.integral {
			if self.integral_data.is_none() {
				let (data, area) = self.integral_rectangles(
					settings.integral_min_x,
					settings.integral_max_x,
					settings.riemann_sum,
					settings.integral_num,
				);

				self.integral_data = Some((
					data.into_iter().map(|(x, y)| Bar::new(x, y)).collect(),
					area,
				));
			}
		} else {
			self.clear_integral();
		}

		let threshold: f64 = resolution / 2.0;
		let x_range = settings.min_x..settings.max_x;

		// Calculates extrema
		if settings.do_extrema && (min_max_changed | self.extrema_data.is_empty()) {
			self.extrema_data = self.newtons_method_helper(threshold, 1, &x_range);
		}

		// Calculates roots
		if settings.do_roots && (min_max_changed | self.root_data.is_empty()) {
			self.root_data = self.newtons_method_helper(threshold, 0, &x_range);
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

		let integral_step =
			(settings.integral_max_x - settings.integral_min_x) / (settings.integral_num as f64);
		debug_assert!(integral_step > 0.0);

		let step = (settings.max_x - settings.min_x) / (settings.plot_width as f64);
		debug_assert!(step > 0.0);

		// Plot back data
		if !self.back_data.is_empty() {
			if self.integral && (step >= integral_step) {
				plot_ui.line(
					self.back_data
						.iter()
						.filter(|value| {
							(value.x > settings.integral_min_x)
								&& (settings.integral_max_x > value.x)
						})
						.cloned()
						.collect::<Vec<PlotPoint>>()
						.to_line()
						.stroke(epaint::Stroke::NONE)
						.color(Color32::from_rgb(4, 4, 255))
						.fill(0.0),
				);
			}
			plot_ui.line(
				self.back_data
					.clone()
					.to_line()
					.stroke(egui::Stroke::new(2.0, main_plot_color)),
			);
		}

		// Plot derivative data
		if self.derivative && !self.derivative_data.is_empty() {
			plot_ui.line(self.derivative_data.clone().to_line().color(Color32::GREEN));
		}

		// Plot extrema points
		if settings.do_extrema && !self.extrema_data.is_empty() {
			plot_ui.points(
				self.extrema_data
					.clone()
					.to_points()
					.color(Color32::YELLOW)
					.radius(5.0), // Radius of points of Extrema
			);
		}

		// Plot roots points
		if settings.do_roots && !self.root_data.is_empty() {
			plot_ui.points(
				self.root_data
					.clone()
					.to_points()
					.color(Color32::LIGHT_BLUE)
					.radius(5.0), // Radius of points of Roots
			);
		}

		if self.nth_derviative && let Some(ref nth_derviative) = self.nth_derivative_data {
			plot_ui.line(
				nth_derviative.clone()
					.to_line()
					.color(Color32::DARK_RED)
			);
		}

		// Plot integral data
		match &self.integral_data {
			Some(integral_data) => {
				if integral_step > step {
					plot_ui.bar_chart(
						BarChart::new(integral_data.0.clone())
							.color(Color32::BLUE)
							.width(integral_step),
					);
				}

				// return value rounded to 8 decimal places
				Some(emath::round_to_decimals(integral_data.1, 8))
			}
			None => None,
		}
	}

	/// Invalidate entire cache
	fn invalidate_whole(&mut self) {
		self.clear_back();
		self.clear_integral();
		self.clear_derivative();
		self.clear_nth();
		self.clear_extrema();
		self.clear_roots();
	}

	/// Invalidate `back` data
	#[inline]
	fn clear_back(&mut self) { self.back_data.clear(); }

	/// Invalidate Integral data
	#[inline]
	fn clear_integral(&mut self) { self.integral_data = None; }

	/// Invalidate Derivative data
	#[inline]
	fn clear_derivative(&mut self) { self.derivative_data.clear(); }

	/// Invalidates `n`th derivative data
	#[inline]
	fn clear_nth(&mut self) { self.nth_derivative_data = None }

	/// Invalidate extrema data
	#[inline]
	fn clear_extrema(&mut self) { self.extrema_data.clear() }

	/// Invalidate root data
	#[inline]
	fn clear_roots(&mut self) { self.root_data.clear() }
}
