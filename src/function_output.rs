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
	/// Creates empty instance of [`FunctionOutput`]
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

/// Tests to make sure invalidation and the default empty state works as
/// expected
#[test]
fn function_output_test() {
	let mut function_output = FunctionOutput::new_empty();
	assert!(function_output.back.is_none());
	assert!(function_output.integral.is_none());
	assert!(function_output.derivative.is_none());
	assert!(function_output.extrema.is_none());
	assert!(function_output.roots.is_none());

	function_output.back = Some(vec![Value::new(0, 0)]);
	function_output.invalidate_back();
	assert!(function_output.back.is_none());

	function_output.integral = Some((vec![Bar::new(0.0, 0.0)], 0.0));
	function_output.invalidate_integral();
	assert!(function_output.integral.is_none());

	function_output.derivative = Some(vec![Value::new(0, 0)]);
	function_output.invalidate_derivative();
	assert!(function_output.derivative.is_none());

	function_output.back = Some(vec![Value::new(0, 0)]);
	function_output.integral = Some((vec![Bar::new(0.0, 0.0)], 0.0));
	function_output.derivative = Some(vec![Value::new(0, 0)]);
	function_output.extrema = Some(vec![Value::new(0, 0)]);
	function_output.roots = Some(vec![Value::new(0, 0)]);

	function_output.invalidate_whole();

	assert!(function_output.back.is_none());
	assert!(function_output.integral.is_none());
	assert!(function_output.derivative.is_none());
	assert!(function_output.extrema.is_none());
	assert!(function_output.roots.is_none());
}
