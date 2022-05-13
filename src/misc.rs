use std::intrinsics::assume;

use eframe::egui::plot::{Line, Points, Value as EguiValue, Values};
use itertools::Itertools;

#[cfg(threading)]
use rayon::prelude::*;

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
pub struct SteppedVector {
	/// Actual data being referenced. HAS to be sorted from minimum to maximum
	data: Vec<f64>,

	/// Minimum value
	min: f64,

	/// Maximum value
	max: f64,

	/// Since all entries in `data` are evenly spaced, this field stores the step between 2 adjacent elements
	step: f64,
}

impl SteppedVector {
	/// Returns `Option<usize>` with index of element with value `x`. and `None` if `x` does not exist in `data`
	pub fn get_index(&self, x: &f64) -> Option<usize> {
		// If `x` is outside range, just go ahead and return `None` as it *shouldn't* be in `data`
		if (x > &self.max) | (&self.min > x) {
			return None;
		}

		if x == &self.min {
			return Some(0);
		}

		if x == &self.max {
			return Some(self.data.len() - 1);
		}

		// Do some math in order to calculate the expected index value
		let possible_i = ((x - self.min) / self.step) as usize;

		// Make sure that the index is valid by checking the data returned vs the actual data (just in case)
		if &self.data[possible_i] == x {
			// It is valid!
			Some(possible_i)
		} else {
			// (For some reason) it wasn't!
			None
		}
	}

	#[allow(dead_code)]
	pub const fn get_min(&self) -> f64 { self.min }

	#[allow(dead_code)]
	pub const fn get_max(&self) -> f64 { self.max }

	#[allow(dead_code)]
	pub fn get_data(&self) -> &Vec<f64> { &self.data }
}

// Convert `&[f64]` into [`SteppedVector`]
impl From<&[f64]> for SteppedVector {
	fn from(data: &[f64]) -> SteppedVector {
		// Ensure data is of correct length
		if data.len() < 2 {
			panic!("SteppedVector: data should have a length longer than 2");
		}

		unsafe {
			assume(data.len() > 2);
			assume(!data.is_empty());
		}

		// length of data subtracted by 1 (represents the maximum index value)
		let max: f64 = data[data.len() - 1]; // The max value should be the first element
		let min: f64 = data[0]; // The minimum value should be the last element

		if min > max {
			panic!("SteppedVector: min is larger than max");
			/*
			tracing::debug!("SteppedVector: min is larger than max, sorting.");
			data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
			max = data[data_i_length];
			min = data[0];
			*/
		}

		// Calculate the step between elements
		let step = (max - min).abs() / (data.len() as f64);

		// Create and return the struct
		SteppedVector {
			data: data.to_vec(),
			min,
			max,
			step,
		}
	}
}

/// Implements traits that are useful when dealing with Vectors of egui's `Value`
pub trait EguiHelper {
	/// Converts to `egui::plot::Line`
	fn to_line(&self) -> Line;

	/// Converts to `egui::plot::Points`
	fn to_points(&self) -> Points;

	/// Converts Vector of Values into vector of tuples
	fn to_tuple(&self) -> Vec<(f64, f64)>;
}

impl EguiHelper for Vec<EguiValue> {
	fn to_line(&self) -> Line { Line::new(Values::from_values(self.clone())) }

	fn to_points(&self) -> Points { Points::new(Values::from_values(self.clone())) }

	fn to_tuple(&self) -> Vec<(f64, f64)> { self.iter().map(|ele| (ele.x, ele.y)).collect() }
}

/// Rounds f64 to `n` decimal places
pub fn decimal_round(x: f64, n: usize) -> f64 {
	let large_number: f64 = 10.0_f64.powf(n as f64); // 10^n

	// round and devide in order to cutoff after the `n`th decimal place
	(x * large_number).round() / large_number
}

/// Helper that assists with using newton's method of finding roots, iterating over data `data`
/// `threshold` is the target accuracy threshold
/// `range` is the range of valid x values (used to stop calculation when the point won't display anyways) `data` is the data to iterate over (a Vector of egui's `Value` struct)
/// `f` is f(x)
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns a Vector of `x` values where roots occur
pub fn newtons_method_helper(
	threshold: &f64, range: &std::ops::Range<f64>, data: &[EguiValue], f: &dyn Fn(f64) -> f64,
	f_1: &dyn Fn(f64) -> f64,
) -> Vec<f64> {
	data.iter()
		.tuple_windows()
		.filter(|(prev, curr)| !prev.y.is_nan() && !curr.y.is_nan())
		.filter(|(prev, curr)| prev.y.signum() != curr.y.signum())
		.map(|(prev, _)| prev.x)
		.map(|start_x| newtons_method(f, f_1, &start_x, range, threshold).unwrap_or(f64::NAN))
		.filter(|x| !x.is_nan())
		.collect()
}

/// `range` is the range of valid x values (used to stop calculation when
/// `f` is f(x)
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns an `Option<f64>` of the x value at which a root occurs
fn newtons_method(
	f: &dyn Fn(f64) -> f64, f_1: &dyn Fn(f64) -> f64, start_x: &f64, range: &std::ops::Range<f64>,
	threshold: &f64,
) -> Option<f64> {
	let mut x1: f64 = *start_x;
	let mut x2: f64;
	let mut fail: bool = false;
	loop {
		x2 = x1 - (f(x1) / f_1(x1));
		if !range.contains(&x2) {
			fail = true;
			break;
		}

		// If below threshold, break
		if (x2 - x1).abs() < *threshold {
			break;
		}

		x1 = x2;
	}

	// If failed, return NaN, which is then filtered out
	match fail {
		true => None,
		false => Some(x1),
	}
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

/// Returns a vector of length `max_i` starting at value `min_x` with resolution of `resolution`
pub fn resolution_helper(max_i: usize, min_x: &f64, resolution: &f64) -> Vec<f64> {
	(0..max_i)
		.map(|x| (x as f64 / resolution) + min_x)
		.collect()
}

/// Returns a vector of length `max_i` starting at value `min_x` with step of `step`
pub fn step_helper(max_i: usize, min_x: &f64, step: &f64) -> Vec<f64> {
	(0..max_i).map(|x| (x as f64 * step) + min_x).collect()
}

const HASH_LENGTH: usize = 8;

#[allow(dead_code)]
pub fn hashed_storage_create(hash: &[u8], data: &[u8]) -> String {
	debug_assert_eq!(hash.len(), HASH_LENGTH);
	debug_assert!(data.len() > 0);

	unsafe {
		assume(data.len() > 0);
		assume(!data.is_empty());
		assume(hash.len() == HASH_LENGTH);
	}

	let new_data = [hash, data].concat();
	debug_assert_eq!(new_data.len(), data.len() + hash.len());

	// cannot use `from_utf8` seems to break on wasm. no clue why
	new_data.iter().map(|b| *b as char).collect::<String>()
}

#[allow(dead_code)]
pub fn hashed_storage_read(data: String) -> (String, Vec<u8>) {
	debug_assert!(data.len() > HASH_LENGTH);
	unsafe {
		assume(!data.is_empty());
		assume(data.len() > 0);
	}

	// can't use data.as_bytes() here for some reason, seems to break on wasm?
	let decoded_1 = data.chars().map(|c| c as u8).collect::<Vec<u8>>();

	let (hash, cached_data) = decoded_1.split_at(8);
	debug_assert_eq!(hash.len(), HASH_LENGTH);
	debug_assert!(cached_data.len() > 0);

	(
		hash.iter().map(|c| *c as char).collect::<String>(),
		cached_data.to_vec(),
	)
}

#[allow(dead_code)]
pub fn format_bytes(bytes: usize) -> String {
	byte_unit::Byte::from_bytes(bytes as u64)
		.get_appropriate_unit(false)
		.to_string()
}
