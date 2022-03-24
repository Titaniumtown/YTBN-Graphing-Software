use eframe::egui::plot::Value as EguiValue;
use itertools::Itertools;
use serde_json::Value as JsonValue;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn dyn_iter<'a, T>(input: &'a Vec<T>) -> impl Iterator<Item = &'a T> { input.iter() }

#[cfg(not(target_arch = "wasm32"))]
pub fn dyn_iter<'a, T>(input: &'a Vec<T>) -> <&'a [T] as IntoParallelIterator>::Iter
where
	&'a [T]: IntoParallelIterator,
{
	input.par_iter()
}

/// `SteppedVector` is used in order to efficiently sort through an ordered
/// `Vec<f64>` Used in order to speedup the processing of cached data when
/// moving horizontally without zoom in `FunctionEntry`. Before this struct, the
/// index was calculated with `.iter().position(....` which was horribly
/// inefficient
pub struct SteppedVector {
	/// Actual data being referenced. HAS to be sorted from minimum to maximum
	data: Vec<f64>,

	/// Minimum value
	min: f64,

	/// Maximum value
	max: f64,

	/// Since all entries in `data` are evenly spaced, this field stores the
	/// step between 2 adjacent elements
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

		if x == self.min {
			return Some(0);
		}

		if x == self.max {
			return Some(self.data.len() - 1);
		}

		// Do some math in order to calculate the expected index value
		let possible_i = ((x - self.min) / self.step) as usize;

		// Make sure that the index is valid by checking the data returned vs the actual
		// data (just in case)
		if self.data[possible_i] == x {
			// It is valid!
			Some(possible_i)
		} else {
			// (For some reason) it wasn't!
			None
		}
	}

	#[allow(dead_code)]
	pub fn get_min(&self) -> f64 { self.min }

	#[allow(dead_code)]
	pub fn get_max(&self) -> f64 { self.max }

	#[allow(dead_code)]
	pub fn get_data(&self) -> Vec<f64> { self.data.clone() }
}

// Convert `Vec<f64>` into `SteppedVector`
impl From<Vec<f64>> for SteppedVector {
	fn from(input_data: Vec<f64>) -> SteppedVector {
		let mut data = input_data;
		// length of data
		let data_length = data.len();

		// Ensure data is of correct length
		if data_length < 2 {
			panic!("SteppedVector: data should have a length longer than 2");
		}

		// length of data subtracted by 1 (represents the maximum index value)
		let data_i_length = data_length - 1;

		let mut max: f64 = data[data_i_length]; // The max value should be the first element
		let mut min: f64 = data[0]; // The minimum value should be the last element

		if min > max {
			tracing::debug!("SteppedVector: min is larger than max, sorting.");
			data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
			max = data[data_i_length];
			min = data[0];
		}

		// Calculate the step between elements
		let step = (max - min).abs() / (data_length as f64);

		// Create and return the struct
		SteppedVector {
			data,
			min,
			max,
			step,
		}
	}
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

/// Helps parsing text data from `text.json`
pub struct SerdeValueHelper {
	value: JsonValue,
}

impl SerdeValueHelper {
	pub fn new(string: &str) -> Self {
		Self {
			value: serde_json::from_str(string).unwrap(),
		}
	}

	/// Parses an array of strings at `self.value[key]` as a multiline string
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

	/// Parses `self.value[key]` as a single line string
	fn parse_singleline(&self, key: &str) -> String { self.value[key].as_str().unwrap().to_owned() }

	/// Used to parse `text.json`
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

/// Rounds f64 to `n` decimal places
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
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns a Vector of `x` values where roots occur
pub fn newtons_method(
	threshold: f64, range: std::ops::Range<f64>, data: Vec<EguiValue>, f: &dyn Fn(f64) -> f64,
	f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	data.iter()
		.tuple_windows()
		.filter(|(prev, curr)| !(prev.y.is_nan() | curr.y.is_nan()))
		.map(|(prev, curr)| {
			if prev.y.signum() != curr.y.signum() {
				// actual start of newton's method
				let x = {
					let mut x1: f64 = prev.x;
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

				return x;
			}
			f64::NAN
		})
		.filter(|x| !x.is_nan())
		.collect()
}

// Returns a vector of length `max_i` starting at value `min_x` with resolution
// of `resolution`
pub fn resolution_helper(max_i: usize, min_x: f64, resolution: f64) -> Vec<f64> {
	(0..max_i)
		.map(|x| (x as f64 / resolution as f64) + min_x)
		.collect()
}

// Returns a vector of length `max_i` starting at value `min_x` with step of
// `step`
pub fn step_helper(max_i: usize, min_x: f64, step: f64) -> Vec<f64> {
	(0..max_i)
		.map(|x| (x as f64 * step as f64) + min_x)
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests SteppedVector to ensure everything works properly (helped me find
	/// a bunch of issues)
	#[test]
	fn stepped_vector_test() {
		let min: i32 = -10;
		let max: i32 = 10;
		let data: Vec<f64> = (min..=max).map(|x| x as f64).collect();
		let len_data = data.len();
		let stepped_vector: SteppedVector = data.into();

		assert_eq!(stepped_vector.get_min(), min as f64);
		assert_eq!(stepped_vector.get_max(), max as f64);

		assert_eq!(stepped_vector.get_index(min as f64), Some(0));
		assert_eq!(stepped_vector.get_index(max as f64), Some(len_data - 1));

		for i in min..=max {
			assert_eq!(
				stepped_vector.get_index(i as f64),
				Some((i + min.abs()) as usize)
			);
		}

		assert_eq!(stepped_vector.get_index((min - 1) as f64), None);
		assert_eq!(stepped_vector.get_index((max + 1) as f64), None);
	}

	/// Ensures decimal_round returns correct values
	#[test]
	fn decimal_round_test() {
		assert_eq!(decimal_round(0.00001, 1), 0.0);
		assert_eq!(decimal_round(0.00001, 2), 0.0);
		assert_eq!(decimal_round(0.00001, 3), 0.0);
		assert_eq!(decimal_round(0.00001, 4), 0.0);
		assert_eq!(decimal_round(0.00001, 5), 0.00001);

		assert_eq!(decimal_round(0.12345, 1), 0.1);
		assert_eq!(decimal_round(0.12345, 2), 0.12);
		assert_eq!(decimal_round(0.12345, 3), 0.123);
		assert_eq!(decimal_round(0.12345, 4), 0.1235); // rounds up
		assert_eq!(decimal_round(0.12345, 5), 0.12345);

		assert_eq!(decimal_round(1.9, 0), 2.0);
		assert_eq!(decimal_round(1.9, 1), 1.9);
	}

	/// Tests `resolution_helper` to make sure it returns expected output
	#[test]
	fn resolution_helper_test() {
		assert_eq!(
			resolution_helper(10, 1.0, 1.0),
			vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
		);

		assert_eq!(
			resolution_helper(5, -2.0, 1.0),
			vec![-2.0, -1.0, 0.0, 1.0, 2.0]
		);

		assert_eq!(resolution_helper(3, -2.0, 1.0), vec![-2.0, -1.0, 0.0]);
	}
}
