use egui::Id;
use egui_plot::{Line, PlotPoint, PlotPoints, Points};
use emath::Pos2;
use getrandom::getrandom;
use itertools::Itertools;
use parsing::FlatExWrapper;

/// Implements traits that are useful when dealing with Vectors of egui's `Value`
pub trait EguiHelper {
	/// Converts to `egui::plot::Values`
	fn to_values(self) -> PlotPoints;

	/// Converts to `egui::plot::Line`
	fn to_line(self) -> Line;

	/// Converts to `egui::plot::Points`
	fn to_points(self) -> Points;

	/// Converts Vector of Values into vector of tuples
	fn to_tuple(self) -> Vec<(f64, f64)>;
}

impl EguiHelper for Vec<PlotPoint> {
	#[inline(always)]
	fn to_values(self) -> PlotPoints {
		PlotPoints::from(unsafe { std::mem::transmute::<Vec<PlotPoint>, Vec<[f64; 2]>>(self) })
	}

	#[inline(always)]
	fn to_line(self) -> Line { Line::new(self.to_values()) }

	#[inline(always)]
	fn to_points(self) -> Points { Points::new(self.to_values()) }

	#[inline(always)]
	fn to_tuple(self) -> Vec<(f64, f64)> {
		unsafe { std::mem::transmute::<Vec<PlotPoint>, Vec<(f64, f64)>>(self) }
	}
}

pub trait Offset {
	fn offset_y(self, y_offset: f32) -> Pos2;
	fn offset_x(self, x_offset: f32) -> Pos2;
}

impl const Offset for Pos2 {
	fn offset_y(self, y_offset: f32) -> Pos2 {
		Pos2 {
			x: self.x,
			y: self.y + y_offset,
		}
	}

	fn offset_x(self, x_offset: f32) -> Pos2 {
		Pos2 {
			x: self.x + x_offset,
			y: self.y,
		}
	}
}

pub const fn create_id(x: u64) -> Id { unsafe { std::mem::transmute::<u64, Id>(x) } }

pub const fn get_u64_id(id: Id) -> u64 { unsafe { std::mem::transmute::<Id, u64>(id) } }

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
	threshold: f64, range: &std::ops::Range<f64>, data: &[PlotPoint], f: &FlatExWrapper,
	f_1: &FlatExWrapper,
) -> Vec<f64> {
	data.iter()
		.tuple_windows()
		.filter(|(prev, curr)| prev.y.is_finite() && curr.y.is_finite())
		.filter(|(prev, curr)| prev.y.signum() != curr.y.signum())
		.map(|(start, _)| start.x)
		.map(|x| newtons_method(f, f_1, x, range, threshold))
		.filter(|x| x.is_some())
		.map(|x| unsafe { x.unwrap_unchecked() })
		.collect()
}

/// `range` is the range of valid x values (used to stop calculation when
/// `f` is f(x)
/// `f_1` is f'(x) aka the derivative of f(x)
/// The function returns an `Option<f64>` of the x value at which a root occurs
pub fn newtons_method(
	f: &FlatExWrapper, f_1: &FlatExWrapper, start_x: f64, range: &std::ops::Range<f64>,
	threshold: f64,
) -> Option<f64> {
	let mut x1: f64 = start_x;
	let mut x2: f64;
	let mut derivative: f64;
	loop {
		derivative = f_1.eval(&[x1]);
		if !derivative.is_finite() {
			return None;
		}

		x2 = x1 - (f.eval(&[x1]) / derivative);
		if !x2.is_finite() | !range.contains(&x2) {
			return None;
		}

		// If below threshold, break
		if (x2 - x1).abs() < threshold {
			return Some(x2);
		}

		x1 = x2;
	}
}

/// Inputs `Vec<Option<T>>` and outputs a `String` containing a pretty representation of the Vector
pub fn option_vec_printer<T: ToString>(data: &[Option<T>]) -> String {
	let formatted: String = data
		.iter()
		.map(|item| match item {
			Some(x) => x.to_string(),
			None => "None".to_owned(),
		})
		.join(", ");

	format!("[{}]", formatted)
}

/// Returns a vector of length `max_i` starting at value `min_x` with step of `step`
pub fn step_helper(max_i: usize, min_x: f64, step: f64) -> Vec<f64> {
	(0..max_i)
		.map(move |x: usize| (x as f64 * step) + min_x)
		.collect()
}

// TODO: use in hovering over points
/// Attempts to see what variable `x` is almost
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
pub type HashBytes = [u8; HASH_LENGTH];

#[allow(dead_code)]
pub fn hashed_storage_create(hashbytes: HashBytes, data: &[u8]) -> String {
	unsafe { std::mem::transmute::<Vec<u8>, String>([hashbytes.to_vec(), data.to_vec()].concat()) }
}

#[allow(dead_code)]
pub fn hashed_storage_read(data: &str) -> Option<(HashBytes, &[u8])> {
	// Make sure data is long enough to decode
	if HASH_LENGTH >= data.len() {
		return None;
	}

	// Transmute data into slice
	let decoded_1: &[u8] = unsafe { std::mem::transmute::<&str, &[u8]>(data) };

	// Return hash and decoded data
	Some((
		unsafe { *(decoded_1[..HASH_LENGTH].as_ptr() as *const HashBytes) },
		&decoded_1[HASH_LENGTH..],
	))
}

/// Creates and returns random u64
pub fn random_u64() -> Result<u64, getrandom::Error> {
	// Buffer of 8 `u8`s that are later merged into one u64
	let mut buf = [0u8; 8];
	// Populate buffer with random values
	getrandom(&mut buf)?;
	// Merge buffer into u64
	Ok(u64::from_be_bytes(buf))
}

include!(concat!(env!("OUT_DIR"), "/valid_chars.rs"));

pub fn is_valid_char(c: char) -> bool { c.is_alphanumeric() | VALID_EXTRA_CHARS.contains(&c) }
