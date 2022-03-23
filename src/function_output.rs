use eframe::egui::{plot::Value, widgets::plot::Bar};

#[derive(Clone)]
pub struct FunctionOutput {
	pub(crate) back: Option<Vec<Value>>,
	pub(crate) integral: Option<(Vec<Bar>, f64)>,
	pub(crate) derivative: Option<Vec<Value>>,
	pub(crate) extrema: Option<Vec<Value>>,
	pub(crate) roots: Option<Vec<Value>>,
}

impl FunctionOutput {
	/// Creates empty instance of `FunctionOutput`
	pub fn new_empty() -> Self {
		Self {
			back: None,
			integral: None,
			derivative: None,
			extrema: None,
			roots: None,
		}
	}

	/// Invalidate all data (setting it all to `None`)
	pub fn invalidate_whole(&mut self) {
		self.back = None;
		self.integral = None;
		self.derivative = None;
		self.extrema = None;
		self.roots = None;
	}

	/// Invalidate `back` data
	pub fn invalidate_back(&mut self) { self.back = None; }

	/// Invalidate Integral data
	pub fn invalidate_integral(&mut self) { self.integral = None; }

	/// Invalidate Derivative data
	pub fn invalidate_derivative(&mut self) { self.derivative = None; }
}
