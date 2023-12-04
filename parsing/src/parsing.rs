use exmex::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct FlatExWrapper {
	func: Option<FlatEx<f64>>,
	func_str: Option<String>,
}

impl FlatExWrapper {
	const EMPTY: FlatExWrapper = FlatExWrapper {
		func: None,
		func_str: None,
	};

	#[inline]
	const fn new(f: FlatEx<f64>) -> Self {
		Self {
			func: Some(f),
			func_str: None,
		}
	}

	#[inline]
	const fn is_none(&self) -> bool { self.func.is_none() }

	#[inline]
	pub fn eval(&self, x: &[f64]) -> f64 {
		self.func
			.as_ref()
			.map(|f| f.eval(x).unwrap_or(f64::NAN))
			.unwrap_or(f64::NAN)
	}

	#[inline]
	fn partial(&self, x: usize) -> Self {
		self.func
			.as_ref()
			.map(|f| f.clone().partial(x).map(Self::new).unwrap_or(Self::EMPTY))
			.unwrap_or(Self::EMPTY)
	}

	#[inline]
	fn get_string(&mut self) -> String {
		match self.func_str {
			Some(ref func_str) => func_str.clone(),
			None => {
				let calculated = self.func.as_ref().map(|f| f.unparse()).unwrap_or("");
				self.func_str = Some(calculated.to_owned());
				calculated.to_owned()
			}
		}
	}

	#[inline]
	fn partial_iter(&self, n: usize) -> Self {
		self.func
			.as_ref()
			.map(|f| {
				f.clone()
					.partial_iter((0..=n).map(|_| 0))
					.map(Self::new)
					.unwrap_or(Self::EMPTY)
			})
			.unwrap_or(Self::EMPTY)
	}
}

impl const Default for FlatExWrapper {
	fn default() -> FlatExWrapper { FlatExWrapper::EMPTY }
}
/// Function that includes f(x), f'(x), f'(x)'s string representation, and f''(x)
#[derive(Clone, PartialEq)]
pub struct BackingFunction {
	/// f(x)
	function: FlatExWrapper,

	/// Temporary cache for nth derivative
	nth_derivative: HashMap<usize, FlatExWrapper>,
}

impl Default for BackingFunction {
	fn default() -> Self { Self::new("").unwrap() }
}

impl BackingFunction {
	pub const fn is_none(&self) -> bool { self.function.is_none() }

	/// Create new [`BackingFunction`] instance
	pub fn new(func_str: &str) -> Result<Self, String> {
		if func_str.is_empty() {
			return Ok(Self {
				function: FlatExWrapper::EMPTY,
				nth_derivative: HashMap::new(),
			});
		}

		let function = FlatExWrapper::new({
			let parse_result = exmex::parse::<f64>(func_str);

			match &parse_result {
				Err(e) => return Err(e.to_string()),
				Ok(ok_result) => {
					let var_names = ok_result.var_names().to_vec();

					if var_names != ["x"] {
						let var_names_not_x: Vec<&String> = var_names
							.iter()
							.filter(|ele| ele != &"x")
							.collect::<Vec<&String>>();

						return Err(format!(
							"Error: invalid variable{}",
							match var_names_not_x.len() {
								1 => String::from(": ") + var_names_not_x[0].as_str(),
								_ => format!("s: {:?}", var_names_not_x),
							}
						));
					}
				}
			}
			unsafe { parse_result.unwrap_unchecked() }
		});

		Ok(Self {
			function,

			nth_derivative: HashMap::new(),
		})
	}

	// TODO rewrite this logic, it's a mess
	pub fn generate_derivative(&mut self, derivative: usize) {
		if derivative == 0 {
			return;
		}

		if !self.nth_derivative.contains_key(&derivative) {
			let new_func = self.function.partial_iter(derivative);
			self.nth_derivative.insert(derivative, new_func.clone());
		}
	}

	pub fn get_function_derivative(&self, derivative: usize) -> &FlatExWrapper {
		if derivative == 0 {
			return &self.function;
		} else {
			return self
				.nth_derivative
				.get(&derivative)
				.unwrap_or(&FlatExWrapper::EMPTY);
		}
	}

	pub fn get(&mut self, derivative: usize, x: f64) -> f64 {
		self.get_function_derivative(derivative).eval(&[x])
	}
}

fn prettyify_function_str(func: &str) -> String {
	let new_str = func.replace("{x}", "x");

	if &new_str == "0/0" {
		"Undefined".to_owned()
	} else {
		new_str
	}
}

// pub const VALID_VARIABLES: [char; 3] = ['x', 'e', 'π'];

/// Case insensitive checks for if `c` is a character used to represent a variable
#[inline]
pub const fn is_variable(c: &char) -> bool {
	let c = c.to_ascii_lowercase();
	(c == 'x') | (c == 'e') | (c == 'π')
}

/// Adds asterisks where needed in a function
pub fn process_func_str(function_in: &str) -> String {
	if function_in.is_empty() {
		return String::new();
	}

	crate::split_function(function_in, crate::SplitType::Multiplication).join("*")
}
