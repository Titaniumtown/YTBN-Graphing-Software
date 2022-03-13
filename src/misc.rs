use std::ops::Range;

use eframe::egui::plot::Value;

// Handles logging based on if the target is wasm (or not) and if `debug_assertions` is enabled or not
cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;
		#[wasm_bindgen]
		extern "C" {
			// `console.log(...)`
			#[wasm_bindgen(js_namespace = console)]
			fn log(s: &str);
		}

		/// Used for logging normal messages
		#[allow(dead_code)]
		pub fn log_helper(s: &str) {
			log(s);
		}

		/// Used for debug messages, only does anything if `debug_assertions` is enabled
		#[allow(dead_code)]
		#[allow(unused_variables)]
		pub fn debug_log(s: &str) {
			#[cfg(debug_assertions)]
			log(s);
		}
	} else {
		/// Used for logging normal messages
		#[allow(dead_code)]
		pub fn log_helper(s: &str) {
			println!("{}", s);
		}

		/// Used for debug messages, only does anything if `debug_assertions` is enabled
		#[allow(dead_code)]
		#[allow(unused_variables)]
		pub fn debug_log(s: &str) {
			#[cfg(debug_assertions)]
			println!("{}", s);
		}
	}
}

/// `SteppedVector` is used in order to efficiently sort through an ordered `Vec<f64>`
/// Used in order to speedup the processing of cached data when moving horizontally without zoom in `FunctionEntry`. Before this struct, the index was calculated with `.iter().position(....` which was horribly inefficient
pub struct SteppedVector {
	// Actual data being referenced. HAS to be sorted from maximum value to minumum
	data: Vec<f64>,

	// Minimum value
	min: f64,

	// Maximum value
	max: f64,

	// Since all entries in `data` are evenly spaced, this field stores the step between 2 adjacent elements
	step: f64,
}

impl SteppedVector {
	/// Returns `Option<usize>` with index of element with value `x`. and `None` if `x` does not exist in `data`
	pub fn get_index(&self, x: f64) -> Option<usize> {
		// if `x` is outside range, just go ahead and return `None` as it *shouldn't* be in `data`
		if (x > self.max) | (self.min > x) {
			return None;
		}

		// Do some math in order to calculate the expected index value
		let possible_i = ((x + self.min) / self.step) as usize;

		// Make sure that the index is valid by checking the data returned vs the actual data (just in case)
		if self.data[possible_i] == x {
			// It is valid!
			Some(possible_i)
		} else {
			// (For some reason) it wasn't!
			None
		}

		// Old (inefficent) code
		/*
		for (i, ele) in self.data.iter().enumerate() {
			if ele > &x {
				return None;
			} else if &x == ele {
				return Some(i);
			}
		}
		None
		*/
	}
}

// Convert `Vec<f64>` into `SteppedVector`
impl From<Vec<f64>> for SteppedVector {
	/// Note: input `data` is assumed to be sorted properly
	/// `data` is a Vector of 64 bit floating point numbers ordered from max -> min
	fn from(data: Vec<f64>) -> SteppedVector {
		let max = data[0]; // The max value should be the first element
		let min = data[data.len() - 1]; // The minimum value should be the last element
		let step = (max - min).abs() / ((data.len() - 1) as f64); // Calculate the step between elements

		// Create and return the struct
		SteppedVector {
			data,
			min,
			max,
			step,
		}
	}
}

// Rounds f64 to specific number of decimal places
pub fn decimal_round(x: f64, n: usize) -> f64 {
	let large_number: f64 = 10.0_f64.powf(n as f64); // 10^n
	(x * large_number).round() / large_number // round and devide in order to cut off after the `n`th decimal place
}

/// Implements newton's method of finding roots.
/// `threshold` is the target accuracy threshold
/// `range` is the range of valid x values (used to stop calculation when the point won't display anyways)
/// `data` is the data to iterate over (a Vector of egui's `Value` struct)
/// `f` is f(x)
/// `f_1` is f'(x)
/// The function returns a Vector of `x` values where roots occur
pub fn newtons_method(
	threshold: f64, range: Range<f64>, data: Vec<Value>, f: &dyn Fn(f64) -> f64,
	f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	let mut output_list: Vec<f64> = Vec::new();
	let mut last_ele_option: Option<Value> = None;
	for ele in data.iter() {
		if last_ele_option.is_none() {
			last_ele_option = Some(*ele);
			continue;
		}

		let last_ele_y = last_ele_option.unwrap().y; // store this here as it's used multiple times

		// If either are NaN, just continue iterating
		if last_ele_y.is_nan() | ele.y.is_nan() {
			continue;
		}

		if last_ele_y.signum() != ele.y.signum() {
			let x = {
				let mut x1: f64 = last_ele_option.unwrap().x;
				let mut x2: f64;
				let mut fail: bool = false;
				loop {
					x2 = x1 - (f(x1) / f_1(x1));
					if !range.contains(&x2) {
						fail = true;
						break;
					}

					// If below threshold, break
					if (x2 - x1).abs() < threshold {
						break;
					}

					x1 = x2;
				}

				// If failed, return NaN, which is then filtered out
				match fail {
					true => f64::NAN,
					false => x1,
				}
			};

			if !x.is_nan() {
				output_list.push(x);
			}
		}
		last_ele_option = Some(*ele);
	}
	output_list
}

/// Parses a json array of strings into a single, multiline string
pub fn parse_value(value: &serde_json::Value) -> String {
	// Create vector of strings
	let string_vector: Vec<&str> = value
		.as_array()
		.unwrap()
		.iter()
		.map(|ele| ele.as_str().unwrap())
		.collect::<Vec<&str>>();

	// Deliminate vector with a new line and return the resulting multiline string
	string_vector
		.iter()
		.fold(String::new(), |s, l| s + l + "\n")
		.trim_end()
		.to_string()
}
