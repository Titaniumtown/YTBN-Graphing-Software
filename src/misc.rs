use std::ops::Range;

use eframe::egui::plot::Value;

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::prelude::*;
		#[wasm_bindgen]
		extern "C" {
			// Use `js_namespace` here to bind `console.log(..)` instead of just
			// `log(..)`
			#[wasm_bindgen(js_namespace = console)]
			fn log(s: &str);
		}

		#[allow(dead_code)]
		pub fn log_helper(s: &str) {
			log(s);
		}

		#[allow(dead_code)]
		#[allow(unused_variables)]
		pub fn debug_log(s: &str) {
			#[cfg(debug_assertions)]
			log(s);
		}
	} else {
		#[allow(dead_code)]
		pub fn log_helper(s: &str) {
			println!("{}", s);
		}

		#[allow(dead_code)]
		#[allow(unused_variables)]
		pub fn debug_log(s: &str) {
			#[cfg(debug_assertions)]
			println!("{}", s);
		}
	}
}

pub struct SteppedVector {
	data: Vec<f64>,
	min: f64,
	max: f64,
	step: f64,
}

impl SteppedVector {
	pub fn get_index(&self, x: f64) -> Option<usize> {
		if (x > self.max) | (self.min > x) {
			return None;
		}

		// Should work....
		let possible_i = ((x + self.min) / self.step) as usize;
		if self.data[possible_i] == x {
			Some(possible_i)
		} else {
			None
		}

		// Not really needed as the above code should handle everything
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

impl From<Vec<f64>> for SteppedVector {
	// Note: input `data` is assumed to be sorted from min to max
	fn from(data: Vec<f64>) -> SteppedVector {
		let max = data[0];
		let min = data[data.len() - 1];
		let step = (max - min).abs() / ((data.len() - 1) as f64);
		SteppedVector {
			data,
			min,
			max,
			step,
		}
	}
}

// Rounds f64 to specific number of digits
pub fn digits_precision(x: f64, digits: usize) -> f64 {
	let large_number: f64 = 10.0_f64.powf(digits as f64);
	(x * large_number).round() / large_number
}

/// Implements newton's method of finding roots.
/// `threshold` is the target accuracy threshold
/// `range` is the range of valid x values (used to stop calculation when the point won't display anyways)
/// `data` is the data to iterate over (a Vector of egui's `Value` struct)
/// `f` is f(x)
/// `f_` is f'(x)
/// The function returns a list of `x` values where roots occur
pub fn newtons_method(
	threshold: f64, range: Range<f64>, data: Vec<Value>, f: &dyn Fn(f64) -> f64,
	f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	let mut output_list: Vec<f64> = Vec::new();
	let mut last_ele: Option<Value> = None;
	for ele in data.iter() {
		if last_ele.is_none() {
			last_ele = Some(*ele);
			continue;
		}

		let last_ele_signum = last_ele.unwrap().y.signum();
		let ele_signum = ele.y.signum();

		// If either are NaN, just continue iterating
		if last_ele_signum.is_nan() | ele_signum.is_nan() {
			continue;
		}

		if last_ele_signum != ele_signum {
			// Do 50 iterations of newton's method, should be more than accurate
			let x = {
				let mut x1: f64 = last_ele.unwrap().x;
				let mut x2: f64;
				let mut fail: bool = false;
				loop {
					x2 = x1 - (f(x1) / f_1(x1));
					if !range.contains(&x2) {
						fail = true;
						break;
					}

					if (x2 - x1).abs() < threshold {
						break;
					}

					x1 = x2;
				}

				match fail {
					true => f64::NAN,
					false => x1,
				}
			};

			if !x.is_nan() {
				output_list.push(x);
			}
		}
		last_ele = Some(*ele);
	}
	output_list
}
