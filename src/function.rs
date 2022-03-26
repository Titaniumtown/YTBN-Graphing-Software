#![allow(clippy::too_many_arguments)] // Clippy, shut

use crate::egui_app::AppSettings;
use crate::function_output::FunctionOutput;
use crate::misc::*;
use crate::parsing::BackingFunction;
use eframe::{egui, epaint};
use egui::{
	plot::{BarChart, PlotUi, Value},
	widgets::plot::Bar,
};
use epaint::Color32;
use std::fmt::{self, Debug};

#[cfg(not(target_arch = "wasm32"))]
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

/// `FunctionEntry` is a function that can calculate values, integrals,
/// derivatives, etc etc
#[derive(Clone)]
pub struct FunctionEntry {
	/// The `BackingFunction` instance that is used to generate `f(x)`, `f'(x)`,
	/// and `f''(x)`
	function: BackingFunction,

	/// Stores a function string (that hasn't been processed via
	/// `process_func_str`) to display to the user
	func_str: String,

	/// Minimum and Maximum values of what do display
	min_x: f64,
	max_x: f64,

	/// output/cached data
	output: FunctionOutput,

	/// If calculating/displayingintegrals are enabled
	pub integral: bool,

	/// If displaying derivatives are enabled (note, they are still calculated
	/// for other purposes)
	pub derivative: bool,
}

impl Default for FunctionEntry {
	/// Creates default FunctionEntry instance (which is empty)
	fn default() -> FunctionEntry {
		FunctionEntry {
			function: BackingFunction::new(""),
			func_str: String::new(),
			min_x: -1.0,
			max_x: 1.0,
			output: FunctionOutput::new_empty(),
			integral: false,
			derivative: false,
		}
	}
}

impl FunctionEntry {
	/// Update function settings
	pub fn update(&mut self, func_str: &str, integral: bool, derivative: bool) {
		// If the function string changes, just wipe and restart from scratch
		if func_str != self.func_str {
			self.func_str = func_str.to_string();
			self.function = BackingFunction::new(func_str);
			self.output.invalidate_whole();
		}

		self.derivative = derivative;
		self.integral = integral;
	}

	/// Creates and does the math for creating all the rectangles under the
	/// graph
	fn integral_rectangles(
		&self, integral_min_x: f64, integral_max_x: f64, sum: Riemann, integral_num: usize,
	) -> (Vec<(f64, f64)>, f64) {
		if integral_min_x.is_nan() {
			panic!("integral_min_x is NaN")
		} else if integral_max_x.is_nan() {
			panic!("integral_max_x is NaN")
		}

		let step = (integral_min_x - integral_max_x).abs() / (integral_num as f64);

		let data2: Vec<(f64, f64)> = dyn_iter(&step_helper(integral_num, integral_min_x, step))
			.map(|x| {
				let step_offset = step * x.signum(); // store the offset here so it doesn't have to be calculated multiple times
				let x2: f64 = x + step_offset;

				let (left_x, right_x) = match x.is_sign_positive() {
					true => (*x, x2),
					false => (x2, *x),
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
			.filter(|(_, y)| !y.is_nan())
			.collect();
		let area = data2.iter().map(|(_, y)| y * step).sum();

		(data2, area)
	}

	/// Returns `func_str`
	pub fn get_func_str(&self) -> &str { &self.func_str }

	fn newtons_method_helper(&self, threshold: f64, derivative_level: usize) -> Option<Vec<Value>> {
		let newtons_method_output: Vec<f64> = match derivative_level {
			0 => newtons_method_helper(
				threshold,
				self.min_x..self.max_x,
				self.output.back.to_owned().unwrap(),
				&|x: f64| self.function.get(x),
				&|x: f64| self.function.get_derivative_1(x),
			),
			1 => newtons_method_helper(
				threshold,
				self.min_x..self.max_x,
				self.output.derivative.to_owned().unwrap(),
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
					.map(|x| (*x, self.function.get(*x)))
					.map(|(x, y)| Value::new(x, y))
					.collect(),
			)
		}
	}

	/// Does the calculations and stores results in `self.output`
	pub fn calculate(
		&mut self, min_x: f64, max_x: f64, width_changed: bool, settings: AppSettings,
	) {
		let resolution: f64 = settings.pixel_width as f64 / (max_x.abs() + min_x.abs());
		let resolution_iter = resolution_helper(settings.pixel_width + 1, min_x, resolution);

		// Makes sure proper arguments are passed when integral is enabled
		if self.integral && settings.integral_changed {
			self.output.invalidate_integral();
		}

		let mut partial_regen = false;
		let min_max_changed = (min_x != self.min_x) | (max_x != self.max_x);

		if width_changed {
			self.output.invalidate_back();
			self.output.invalidate_derivative();
			self.min_x = min_x;
			self.max_x = max_x;
		} else if min_max_changed && self.output.back.is_some() {
			partial_regen = true;

			let back_cache = self.output.back.as_ref().unwrap();

			let x_data: SteppedVector = back_cache
				.iter()
				.map(|ele| ele.x)
				.collect::<Vec<f64>>()
				.into();

			let back_data: Vec<Value> = dyn_iter(&resolution_iter)
				.cloned()
				.map(|x| {
					if let Some(i) = x_data.get_index(x) {
						back_cache[i]
					} else {
						Value::new(x, self.function.get(x))
					}
				})
				.collect();
			assert_eq!(back_data.len(), settings.pixel_width + 1);
			self.output.back = Some(back_data);

			let derivative_cache = self.output.derivative.as_ref().unwrap();
			let new_derivative_data: Vec<Value> = dyn_iter(&resolution_iter)
				.map(|x| {
					if let Some(i) = x_data.get_index(*x) {
						derivative_cache[i]
					} else {
						Value::new(*x, self.function.get_derivative_1(*x))
					}
				})
				.collect();

			assert_eq!(new_derivative_data.len(), settings.pixel_width + 1);

			self.output.derivative = Some(new_derivative_data);
		} else {
			self.output.invalidate_back();
			self.output.invalidate_derivative();
		}

		self.min_x = min_x;
		self.max_x = max_x;

		let threshold: f64 = resolution / 2.0;

		if !partial_regen {
			self.output.back = Some({
				if self.output.back.is_none() {
					let data: Vec<Value> = dyn_iter(&resolution_iter)
						.map(|x| Value::new(*x, self.function.get(*x)))
						.collect();
					assert_eq!(data.len(), settings.pixel_width + 1);

					self.output.back = Some(data);
				}

				self.output.back.as_ref().unwrap().clone()
			});

			self.output.derivative = {
				if self.output.derivative.is_none() {
					let data: Vec<Value> = dyn_iter(&resolution_iter)
						.map(|x| Value::new(*x, self.function.get_derivative_1(*x)))
						.collect();
					assert_eq!(data.len(), settings.pixel_width + 1);
					self.output.derivative = Some(data);
				}

				Some(self.output.derivative.as_ref().unwrap().clone())
			};
		}

		self.output.integral = match self.integral {
			true => {
				if self.output.integral.is_none() {
					let (data, area) = self.integral_rectangles(
						settings.integral_min_x,
						settings.integral_max_x,
						settings.sum,
						settings.integral_num,
					);
					self.output.integral =
						Some((data.iter().map(|(x, y)| Bar::new(*x, *y)).collect(), area));
				}

				let cache = self.output.integral.as_ref().unwrap();
				Some((cache.0.clone(), cache.1))
			}
			false => None,
		};

		// Calculates extrema
		if settings.extrema && (min_max_changed | self.output.extrema.is_none()) {
			self.output.extrema = self.newtons_method_helper(threshold, 1);
		}

		// Calculates roots
		if settings.roots && (min_max_changed | self.output.roots.is_none()) {
			self.output.roots = self.newtons_method_helper(threshold, 0);
		}
	}

	/// Calculates and displays the function on PlotUI `plot_ui`
	pub fn display(&self, plot_ui: &mut PlotUi, settings: AppSettings) -> Option<f64> {
		// self.calculate(min_x, max_x, width_changed, settings);

		let func_str = self.get_func_str();
		let derivative_str = self.function.get_derivative_str();
		let step = (settings.integral_min_x - settings.integral_max_x).abs()
			/ (settings.integral_num as f64);
		// Plot back data
		plot_ui.line(
			vec_tuple_to_line(self.output.back.clone().unwrap())
				.color(Color32::RED)
				.name(func_str),
		);

		// Plot derivative data
		if self.derivative {
			if let Some(derivative_data) = self.output.derivative.clone() {
				plot_ui.line(
					vec_tuple_to_line(derivative_data)
						.color(Color32::GREEN)
						.name(derivative_str),
				);
			}
		}

		// Plot extrema points
		if settings.extrema {
			if let Some(extrema_data) = self.output.extrema.clone() {
				plot_ui.points(
					vec_tuple_to_points(extrema_data)
						.color(Color32::YELLOW)
						.name("Extrema")
						.radius(5.0), // Radius of points of Extrema
				);
			}
		}

		// Plot roots points
		if settings.roots {
			if let Some(roots_data) = self.output.roots.clone() {
				plot_ui.points(
					vec_tuple_to_points(roots_data)
						.color(Color32::LIGHT_BLUE)
						.name("Root")
						.radius(5.0), // Radius of points of Roots
				);
			}
		}

		// Plot integral data
		if let Some(integral_data) = self.output.integral.clone() {
			plot_ui.bar_chart(
				BarChart::new(integral_data.0)
					.color(Color32::BLUE)
					.width(step),
			);

			// return value rounded to 8 decimal places
			Some(crate::misc::decimal_round(integral_data.1, 8))
		} else {
			None
		}
	}

	#[cfg(test)]
	pub fn tests(
		&mut self, settings: AppSettings, back_target: Vec<(f64, f64)>,
		derivative_target: Vec<(f64, f64)>, area_target: f64, min_x: f64, max_x: f64,
	) {
		{
			self.calculate(min_x, max_x, true, settings);
			let settings = settings;
			let back_target = back_target;
			assert!(self.output.back.is_some());
			let back_data = self.output.back.as_ref().unwrap().clone();
			assert_eq!(back_data.len(), settings.pixel_width + 1);
			let back_vec_tuple = back_data.to_tuple();
			assert_eq!(back_vec_tuple, back_target);

			assert!(self.integral);
			assert!(self.derivative);

			assert_eq!(self.output.roots.is_some(), settings.roots);
			assert_eq!(self.output.extrema.is_some(), settings.extrema);
			assert!(self.output.derivative.is_some());
			assert!(self.output.integral.is_some());

			assert_eq!(
				self.output.derivative.as_ref().unwrap().to_tuple(),
				derivative_target
			);

			assert_eq!(self.output.integral.clone().unwrap().1, area_target);
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
		crate::egui_app::AppSettings {
			help_open: false,
			info_open: false,
			show_side_panel: false,
			sum,
			integral_min_x,
			integral_max_x,
			integral_changed: true,
			integral_num,
			dark_mode: false,
			extrema: false,
			roots: false,
			pixel_width,
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
	fn left_function_test() {
		let area_target = 0.9600000000000001;

		do_test(Riemann::Left, area_target);
	}

	#[test]
	fn middle_function_test() {
		let area_target = 0.92;

		do_test(Riemann::Middle, area_target);
	}

	#[test]
	fn right_function_test() {
		let area_target = 0.8800000000000001;

		do_test(Riemann::Right, area_target);
	}
}
