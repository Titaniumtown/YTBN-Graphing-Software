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
