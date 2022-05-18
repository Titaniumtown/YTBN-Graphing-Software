use std::intrinsics::assume;

use egui::plot::{Line, Points, Value, Values};
use itertools::Itertools;

#[cfg(not(threading))]
#[inline]
pub fn dyn_iter<'a, T>(input: &'a [T]) -> impl Iterator<Item = &'a T>
where
	&'a [T]: IntoIterator,
{
	input.iter()
}

#[cfg(threading)]
#[inline]
pub fn dyn_iter<'a, I>(input: &'a I) -> <&'a I as IntoParallelIterator>::Iter
where
	&'a I: IntoParallelIterator,
{
	use rayon::prelude::*;

	input.par_iter()
}

#[cfg(not(threading))]
#[inline]
pub fn dyn_mut_iter<'a, T>(input: &'a mut [T]) -> impl Iterator<Item = &'a mut T>
where
	&'a mut [T]: IntoIterator,
{
	input.iter_mut()
}

#[cfg(threading)]
#[inline]
pub fn dyn_mut_iter<'a, I>(input: &'a mut I) -> <&'a mut I as IntoParallelIterator>::Iter
where
	&'a mut I: IntoParallelIterator,
{
	use rayon::prelude::*;
	input.par_iter_mut()
}

pub struct FunctionHelper<'a> {
	#[cfg(threading)]
	f: async_lock::Mutex<Box<dyn Fn(f64, f64) -> f64 + 'a + Sync + Send>>,

	#[cfg(not(threading))]
	f: Box<dyn Fn(f64, f64) -> f64 + 'a>,
}

impl<'a> FunctionHelper<'a> {
	#[cfg(threading)]
	pub fn new(f: impl Fn(f64, f64) -> f64 + 'a) -> FunctionHelper<'a> {
		FunctionHelper {
			f: async_lock::Mutex::new(Box::new(f)),
		}
	}

	#[cfg(not(threading))]
	pub fn new(f: impl Fn(f64, f64) -> f64 + 'a) -> FunctionHelper<'a> {
		FunctionHelper { f: Box::new(f) }
	}

	#[cfg(threading)]
	pub async fn get(&self, x: f64, x1: f64) -> f64 { (self.f.lock().await)(x, x1) }

	#[cfg(not(threading))]
	pub fn get(&self, x: f64, x1: f64) -> f64 { (self.f)(x, x1) }
}

/// [`SteppedVector`] is used in order to efficiently sort through an ordered
/// `Vec<f64>` Used in order to speedup the processing of cached data when
/// moving horizontally without zoom in `FunctionEntry`. Before this struct, the
/// index was calculated with `.iter().position(....` which was horribly
/// inefficient
pub struct SteppedVector<'a> {
	/// Actual data being referenced. HAS to be sorted from minimum to maximum
	data: &'a [f64],

	/// Since all entries in `data` are evenly spaced, this field stores the step between 2 adjacent elements
	step: f64,
}

impl<'a> SteppedVector<'a> {
	/// Returns `Option<usize>` with index of element with value `x`. and `None` if `x` does not exist in `data`
	pub fn get_index(&self, x: f64) -> Option<usize> {
		debug_assert!(!x.is_nan());
		debug_assert!(self.step > 0.0);
		debug_assert!(self.step.is_sign_positive());
		debug_assert!(self.data.len() >= 2);

		unsafe {
			assume(!self.step.is_nan());
			assume(self.step > 0.0);
			assume(self.step.is_sign_positive());
			assume(self.data.len() >= 2);
		}

		let max = self.get_max();
		if &x > max {
			return None;
		}

		let min = self.get_min();
		if min > &x {
			return None;
		}

		if &x == min {
			return Some(0);
		} else if &x == max {
			return Some(self.data.len() - 1);
		}

		// Do some math in order to calculate the expected index value
		let possible_i = ((x - min).abs() / self.step) as usize;

		// Make sure that the index is valid by checking the data returned vs the actual data (just in case)
		if self.data.get(possible_i) == Some(&x) {
			// It is valid!
			Some(possible_i)
		} else {
			// (For some reason) it wasn't!
			None
		}
	}

	#[inline]
	pub const fn get_min(&self) -> &f64 {
		debug_assert!(self.data.len() >= 2);
		unsafe {
			assume(!self.data.is_empty());
			self.data.get_unchecked(0)
		}
	}

	#[inline]
	pub const fn get_max(&self) -> &f64 {
		debug_assert!(self.data.len() >= 2);
		unsafe {
			assume(!self.data.is_empty());
			self.data.last().unwrap_unchecked()
		}
	}

	#[allow(dead_code)]
	pub fn get_data(&self) -> &'a [f64] { self.data }
}

// Convert `&[f64]` into [`SteppedVector`]
impl<'a> From<&'a [f64]> for SteppedVector<'a> {
	fn from(data: &'a [f64]) -> SteppedVector {
		// Ensure data is of correct length
		debug_assert!(data.len() > 2);

		unsafe {
			assume(data.len() > 2);
			assume(!data.is_empty());
		}

		// length of data subtracted by 1 (represents the maximum index value)
		let max: f64 = data[data.len() - 1]; // The max value should be the last element
		let min: f64 = data[0]; // The minimum value should be the first element

		debug_assert!(max > min);

		unsafe {
			assume(max > min);
		}

		// Calculate the step between elements
		let step = (max - min).abs() / (data.len() as f64);

		debug_assert!(step.is_sign_positive());
		debug_assert!(step.is_finite());

		// Create and return the struct
		SteppedVector { data, step }
	}
}

/// Implements traits that are useful when dealing with Vectors of egui's `Value`
pub trait EguiHelper {
	/// Converts to `egui::plot::Values`
	fn to_values(self) -> Values;

	/// Converts to `egui::plot::Line`
	fn to_line(self) -> Line;

	/// Converts to `egui::plot::Points`
	fn to_points(self) -> Points;

	/// Converts Vector of Values into vector of tuples
	fn to_tuple(self) -> Vec<(f64, f64)>;
}

impl EguiHelper for Vec<Value> {
	#[inline(always)]
	fn to_values(self) -> Values { Values::from_values(self) }

	#[inline(always)]
	fn to_line(self) -> Line { Line::new(self.to_values()) }

	#[inline(always)]
	fn to_points(self) -> Points { Points::new(self.to_values()) }

	#[inline(always)]
	fn to_tuple(self) -> Vec<(f64, f64)> {
		// self.iter().map(|ele| (ele.x, ele.y)).collect()
		unsafe { std::mem::transmute::<Vec<Value>, Vec<(f64, f64)>>(self) }
	}
}

/*
/// Rounds f64 to `n` decimal places
pub fn decimal_round(x: f64, n: usize) -> f64 {
	let large_number: f64 = 10.0_f64.powf(n as f64); // 10^n

	// round and devide in order to cutoff after the `n`th decimal place
	(x * large_number).round() / large_number
}
*/

/// Helper that assists with using newton's method of finding roots, iterating over data `data`
/// `threshold` is the target accuracy threshold
/// `range` is the range of valid x values (used to stop calculation when the point won't display anyways) `data` is the data to iterate over (a Vector of egui's `Value` struct)
/// `f` is f(x)
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns a Vector of `x` values where roots occur
pub fn newtons_method_helper(
	threshold: &f64, range: &std::ops::Range<f64>, data: &[Value], f: &dyn Fn(f64) -> f64,
	f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	data.iter()
		.tuple_windows()
		.filter(|(prev, curr)| prev.y.is_finite() && curr.y.is_finite())
		.filter(|(prev, curr)| prev.y.signum() != curr.y.signum())
		.map(|(start, _)| newtons_method(f, f_1, &start.x, range, threshold))
		.filter(|x| x.is_some())
		.map(|x| unsafe { x.unwrap_unchecked() })
		.collect()
}

/// `range` is the range of valid x values (used to stop calculation when
/// `f` is f(x)
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns an `Option<f64>` of the x value at which a root occurs
pub fn newtons_method(
	f: &dyn Fn(f64) -> f64, f_1: &dyn Fn(f64) -> f64, start_x: &f64, range: &std::ops::Range<f64>,
	threshold: &f64,
) -> Option<f64> {
	let mut x1: f64 = *start_x;
	let mut x2: f64;
	let mut derivative: f64;
	loop {
		derivative = f_1(x1);
		if !derivative.is_finite() {
			return None;
		}

		x2 = x1 - (f(x1) / derivative);
		if !x2.is_finite() | !range.contains(&x2) {
			return None;
		}

		// If below threshold, break
		if (x2 - x1).abs() < *threshold {
			break;
		}

		x1 = x2;
	}

	// return x2 as loop breaks before x1 is set to x2
	Some(x2)
}

/// Inputs `Vec<Option<T>>` and outputs a `String` containing a pretty representation of the Vector
pub fn option_vec_printer<T: ToString>(data: &[Option<T>]) -> String
where
	T: ToString,
{
	let max_i: i32 = (data.len() as i32) - 1;
	[
		"[",
		&data
			.iter()
			.map(|x| {
				x.as_ref()
					.map(|x_1| x_1.to_string())
					.unwrap_or_else(|| "None".to_owned())
			})
			.enumerate()
			.map(|(i, x)| {
				// Add comma and space if needed
				match max_i > i as i32 {
					true => x + ", ",
					false => x,
				}
			})
			.collect::<Vec<String>>()
			.concat(),
		"]",
	]
	.concat()
}

/// Returns a vector of length `max_i` starting at value `min_x` with step of `step`
pub fn step_helper(max_i: usize, min_x: &f64, step: &f64) -> Vec<f64> {
	(0..max_i).map(|x| (x as f64 * step) + min_x).collect()
}

/// Attempts to see what variable `x` is almost
// TODO: use in hovering over points
#[allow(dead_code)]
pub fn almost_variable(x: f64) -> Option<char> {
	const EPSILON: f32 = f32::EPSILON * 2.0;
	if emath::almost_equal(x as f32, std::f32::consts::E, EPSILON) {
		Some('e')
	} else if emath::almost_equal(x as f32, std::f32::consts::PI, EPSILON) {
		Some('π')
	} else {
		None
	}
}

pub const HASH_LENGTH: usize = 8;

/// Represents bytes used to represent hash info
type HashBytes = [u8; HASH_LENGTH];

#[allow(dead_code)]
pub fn hashed_storage_create(hash: HashBytes, data: &[u8]) -> String {
	// cannot use `from_utf8` seems to break on wasm. no clue why
	[&hash, data]
		.concat()
		.iter()
		.map(|b| *b as char)
		.collect::<String>()
}

#[allow(dead_code)]
pub fn hashed_storage_read(data: &str) -> Option<(HashBytes, Vec<u8>)> {
	if HASH_LENGTH >= data.len() {
		return None;
	}

	unsafe {
		assume(!data.is_empty());
		assume(data.len() > HASH_LENGTH);
	}

	// can't use data.as_bytes() here for some reason, seems to break on wasm?
	let decoded_1: Vec<u8> = data.chars().map(|c| c as u8).collect::<Vec<u8>>();

	let hash: HashBytes = unsafe { *(decoded_1[..HASH_LENGTH].as_ptr() as *const HashBytes) };
	let cached_data = decoded_1[HASH_LENGTH..].to_vec();

	debug_assert!(!cached_data.is_empty());

	Some((hash, cached_data))
}
