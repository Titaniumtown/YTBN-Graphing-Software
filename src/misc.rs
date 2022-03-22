use std::ops::Range;

// Handles logging based on if the target is wasm (or not) and if
// `debug_assertions` is enabled or not
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

/// `SteppedVector` is used in order to efficiently sort through an ordered
/// `Vec<f64>` Used in order to speedup the processing of cached data when
/// moving horizontally without zoom in `FunctionEntry`. Before this struct, the
/// index was calculated with `.iter().position(....` which was horribly
/// inefficient
pub struct SteppedVector {
	// Actual data being referenced. HAS to be sorted from maximum value to minumum
	data: Vec<f64>,

	// Minimum value
	min: f64,

	// Maximum value
	max: f64,

	// Since all entries in `data` are evenly spaced, this field stores the step between 2 adjacent
	// elements
	step: f64,
}

impl SteppedVector {
	/// Returns `Option<usize>` with index of element with value `x`. and `None`
	/// if `x` does not exist in `data`
	pub fn get_index(&self, x: f64) -> Option<usize> {
		// if `x` is outside range, just go ahead and return `None` as it *shouldn't* be
		// in `data`
		if (x > self.max) | (self.min > x) {
			return None;
		}

		// Do some math in order to calculate the expected index value
		let possible_i = ((x + self.min) / self.step) as usize;

		// Make sure that the index is valid by checking the data returned vs the actual
		// data (just in case)
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
	fn from(input_data: Vec<f64>) -> SteppedVector {
		let mut data = input_data;
		// length of data
		let data_length = data.len();
		// length of data subtracted by 1 (represents the maximum index value)
		let data_i_length = data.len() - 1;

		// Ensure data is of correct length
		if data_length < 2 {
			panic!("SteppedVector: data should have a length longer than 2");
		}

		let mut max: f64 = data[0]; // The max value should be the first element
		let mut min: f64 = data[data_i_length]; // The minimum value should be the last element

		// if min is bigger than max, sort the input data
		if min > max {
			data.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
			max = data[0];
			min = data[data_i_length];
		}

		let step = (max - min).abs() / (data_i_length as f64); // Calculate the step between elements

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
	(x * large_number).round() / large_number // round and devide in order to cut
	                                      // off after the `n`th decimal place
}

/// Implements newton's method of finding roots.
/// `threshold` is the target accuracy threshold
/// `range` is the range of valid x values (used to stop calculation when the
/// point won't display anyways) `data` is the data to iterate over (a Vector of
/// egui's `Value` struct) `f` is f(x)
/// `f_1` is f'(x)
/// The function returns a Vector of `x` values where roots occur
pub fn newtons_method(
	threshold: f64, range: Range<f64>, data: Vec<eframe::egui::plot::Value>,
	f: &dyn Fn(f64) -> f64, f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	let mut output_list: Vec<f64> = Vec::new();
	let mut last_ele_option: Option<eframe::egui::plot::Value> = None;
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

#[derive(PartialEq, Debug)]
pub struct JsonFileOutput {
	pub help_expr: String,
	pub help_vars: String,
	pub help_panel: String,
	pub help_function: String,
	pub help_other: String,
	pub license_info: String,
}

pub struct SerdeValueHelper {
	value: serde_json::Value,
}

impl SerdeValueHelper {
	pub fn new(string: &str) -> Self {
		Self {
			value: serde_json::from_str(string).unwrap(),
		}
	}

	fn parse_multiline(&self, key: &str) -> String {
		(&self.value[key])
			.as_array()
			.unwrap()
			.iter()
			.map(|ele| ele.as_str().unwrap())
			.fold(String::new(), |s, l| s + l + "\n")
			.trim_end()
			.to_owned()
	}

	fn parse_singleline(&self, key: &str) -> String { self.value[key].as_str().unwrap().to_owned() }

	pub fn parse_values(&self) -> JsonFileOutput {
		JsonFileOutput {
			help_expr: self.parse_multiline("help_expr"),
			help_vars: self.parse_multiline("help_vars"),
			help_panel: self.parse_multiline("help_panel"),
			help_function: self.parse_multiline("help_function"),
			help_other: self.parse_multiline("help_other"),
			license_info: self.parse_singleline("license_info"),
		}
	}
}
