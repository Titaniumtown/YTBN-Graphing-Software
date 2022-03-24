#![allow(clippy::too_many_arguments)] // Clippy, shut

use crate::egui_app::AppSettings;
use crate::function_output::FunctionOutput;
use crate::misc::{dyn_iter, newtons_method_helper, resolution_helper, step_helper, SteppedVector};
use crate::parsing::BackingFunction;
use eframe::{egui, epaint};
use egui::{
	plot::{BarChart, Line, PlotUi, Points, Value, Values},
	widgets::plot::Bar,
};
use epaint::Color32;
use std::fmt::{self, Debug};

#[cfg(not(target_arch = "wasm32"))]
use rayon::iter::ParallelIterator;

/// Represents the possible variations of Riemann Sums
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RiemannSum {
	Left,
	Middle,
	Right,
}

impl fmt::Display for RiemannSum {
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
	pub fn update(&mut self, func_str: String, integral: bool, derivative: bool) {
		// If the function string changes, just wipe and restart from scratch
		if func_str != self.func_str {
			self.func_str = func_str.clone();
			self.function = BackingFunction::new(&func_str);
			self.output.invalidate_whole();
		}

		self.derivative = derivative;
		self.integral = integral;
	}

	/// Creates and does the math for creating all the rectangles under the
	/// graph
	fn integral_rectangles(
		&self, integral_min_x: f64, integral_max_x: f64, sum: RiemannSum, integral_num: usize,
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
					RiemannSum::Left => self.function.get(left_x),
					RiemannSum::Right => self.function.get(right_x),
					RiemannSum::Middle => {
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

	/// Calculates and displays the function on PlotUI `plot_ui`
	pub fn display(
		&mut self, plot_ui: &mut PlotUi, min_x: f64, max_x: f64, pixel_width: usize,
		width_changed: bool, settings: AppSettings,
	) -> f64 {
		let resolution: f64 = pixel_width as f64 / (max_x.abs() + min_x.abs());
		let resolution_iter = resolution_helper(pixel_width + 1, min_x, resolution);

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
			assert_eq!(back_data.len(), pixel_width + 1);
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

			assert_eq!(new_derivative_data.len(), pixel_width + 1);

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
					assert_eq!(data.len(), pixel_width + 1);

					self.output.back = Some(data);
				}

				self.output.back.as_ref().unwrap().clone()
			});

			self.output.derivative = {
				if self.output.derivative.is_none() {
					let data: Vec<Value> = dyn_iter(&resolution_iter)
						.map(|x| Value::new(*x, self.function.get_derivative_1(*x)))
						.collect();
					assert_eq!(data.len(), pixel_width + 1);
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

		let func_str = self.get_func_str();
		let derivative_str = self.function.get_derivative_str();
		let step = (settings.integral_min_x - settings.integral_max_x).abs()
			/ (settings.integral_num as f64);
		// Plot back data
		plot_ui.line(
			Line::new(Values::from_values(self.output.back.clone().unwrap()))
				.color(Color32::RED)
				.name(func_str),
		);

		// Plot derivative data
		if self.derivative {
			if let Some(derivative_data) = self.output.derivative.clone() {
				plot_ui.line(
					Line::new(Values::from_values(derivative_data))
						.color(Color32::GREEN)
						.name(derivative_str),
				);
			}
		}

		// Plot extrema points
		if settings.extrema {
			if let Some(extrema_data) = self.output.extrema.clone() {
				plot_ui.points(
					Points::new(Values::from_values(extrema_data))
						.color(Color32::YELLOW)
						.name("Extrema")
						.radius(5.0),
				);
			}
		}

		// Plot roots points
		if settings.roots {
			if let Some(roots_data) = self.output.roots.clone() {
				plot_ui.points(
					Points::new(Values::from_values(roots_data))
						.color(Color32::LIGHT_BLUE)
						.name("Root")
						.radius(5.0),
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
			crate::misc::decimal_round(integral_data.1, 8)
		} else {
			f64::NAN // return NaN if integrals are disabled
		}
	}
}

/*
#[cfg(test)]
mod tests {
	use super::*;

	fn verify_function(
		integral_num: usize, pixel_width: usize, function: &mut FunctionEntry,
		back_values_target: Vec<(f64, f64)>, area_target: f64,
	) {
		{
			let (back_values, bars, derivative) = function.run_back(-1.0, 1.0);
			assert!(derivative.is_some());
			assert!(bars.is_none());
			assert_eq!(back_values.len(), pixel_width);
			let back_values_tuple: Vec<(f64, f64)> =
				back_values.iter().map(|ele| (ele.x, ele.y)).collect();
			assert_eq!(back_values_tuple, back_values_target);
		}

		{
			*function = function.clone().integral(true);
			let (back_values, bars, derivative) = function.run_back(-1.0, 1.0);
			assert!(derivative.is_some());
			assert!(bars.is_some());
			assert_eq!(back_values.len(), pixel_width);

			assert_eq!(bars.clone().unwrap().1, area_target);

			let vec_bars = bars.unwrap().0;
			assert_eq!(vec_bars.len(), integral_num);

			let back_values_tuple: Vec<(f64, f64)> =
				back_values.iter().map(|ele| (ele.x, ele.y)).collect();
			assert_eq!(back_values_tuple, back_values_target);
		}

		{
			let (back_values, bars, derivative) = function.run_back(-1.0, 1.0);
			assert!(derivative.is_some());

			assert!(bars.is_some());
			assert_eq!(back_values.len(), pixel_width);
			assert_eq!(bars.clone().unwrap().1, area_target);
			let bars_unwrapped = bars.unwrap();

			assert_eq!(bars_unwrapped.0.iter().len(), integral_num);
		}
	}

	#[test]
	fn left_function_test() {
		let integral_num = 10;
		let pixel_width = 10;

		let mut function = FunctionEntry::default()
			.update_riemann(RiemannSum::Left)
			.pixel_width(pixel_width)
			.integral_num(integral_num);

		let back_values_target = vec![
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
		];

		let area_target = 0.9600000000000001;

		verify_function(
			integral_num,
			pixel_width,
			&mut function,
			back_values_target,
			area_target,
		);
	}

	#[test]
	fn middle_function_test() {
		let integral_num = 10;
		let pixel_width = 10;

		let mut function = FunctionEntry::default()
			.update_riemann(RiemannSum::Middle)
			.pixel_width(pixel_width)
			.integral_num(integral_num)
			.integral_bounds(-1.0, 1.0);

		let back_values_target = vec![
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
		];

		let area_target = 0.92;

		verify_function(
			integral_num,
			pixel_width,
			&mut function,
			back_values_target,
			area_target,
		);
	}

	#[test]
	fn right_function_test() {
		let integral_num = 10;
		let pixel_width = 10;

		let mut function = FunctionEntry::default()
			.update_riemann(RiemannSum::Right)
			.pixel_width(pixel_width)
			.integral_num(integral_num)
			.integral_bounds(-1.0, 1.0);

		let back_values_target = vec![
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
		];

		let area_target = 0.8800000000000001;

		verify_function(
			integral_num,
			pixel_width,
			&mut function,
			back_values_target,
			area_target,
		);
	}
}
*/
