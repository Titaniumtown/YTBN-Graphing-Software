use exmex::prelude::*;

#[derive(Clone, PartialEq)]
pub(crate) struct FlatExWrapper {
	func: Option<FlatEx<f64>>,
}

impl FlatExWrapper {
	const EMPTY: FlatExWrapper = FlatExWrapper { func: None };

	#[inline]
	const fn new(f: FlatEx<f64>) -> Self { Self { func: Some(f) } }

	#[inline]
	const fn is_none(&self) -> bool { self.func.is_none() }

	#[inline]
	fn eval(&self, x: &[f64]) -> f64 {
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
	fn get_string(&self) -> &str { self.func.as_ref().map(|f| f.unparse()).unwrap_or("") }

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

	/// f'(x)
	derivative_1: FlatExWrapper,

	/// Mathematical representation of f'(x)
	derivative_1_str: String,

	/// f''(x)
	derivative_2: FlatExWrapper,

	/// Temporary cache for nth derivative
	nth_derivative: Option<(usize, FlatExWrapper, String)>,
}

impl BackingFunction {
	/// Empty [`BackingFunction`] instance
	pub const EMPTY: BackingFunction = BackingFunction {
		function: FlatExWrapper::EMPTY,
		derivative_1: FlatExWrapper::EMPTY,
		derivative_1_str: String::new(),
		derivative_2: FlatExWrapper::EMPTY,
		nth_derivative: None,
	};

	pub const fn is_none(&self) -> bool { self.function.is_none() }

	/// Create new [`BackingFunction`] instance
	pub fn new(func_str: &str) -> Result<Self, String> {
		if func_str.is_empty() {
			return Ok(Self::EMPTY);
		}

		let function = FlatExWrapper::new({
			let parse_result = exmex::parse::<f64>(func_str);

			match &parse_result {
				Err(e) => return Err(e.to_string()),
				Ok(_) => {
					let var_names = unsafe { parse_result.as_ref().unwrap_unchecked() }
						.var_names()
						.to_vec();

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

		let derivative_1 = function.partial(0);

		let derivative_1_str = prettyify_function_str(derivative_1.get_string());

		let derivative_2 = derivative_1.partial(0);

		Ok(Self {
			function,
			derivative_1,
			derivative_1_str,
			derivative_2,
			nth_derivative: None,
		})
	}

	/// Returns Mathematical representation of the function's derivative
	pub fn get_derivative_str(&self) -> &str { &self.derivative_1_str }

	/// Calculate f(x)
	pub fn get(&self, x: f64) -> f64 { self.function.eval(&[x]) }

	/// Calculate f'(x)
	pub fn get_derivative_1(&self, x: f64) -> f64 { self.derivative_1.eval(&[x]) }

	/// Calculate f''(x)
	pub fn get_derivative_2(&self, x: f64) -> f64 { self.derivative_2.eval(&[x]) }

	/// Get string relating to the nth derivative
	pub fn get_nth_derivative_str(&self) -> &str {
		self.nth_derivative
			.as_ref()
			.map(|a| a.2.as_str())
			.unwrap_or("")
	}

	pub fn get_nth_derivative(&mut self, n: usize, x: f64) -> f64 {
		match n {
			0 => self.get(x),
			1 => self.get_derivative_1(x),
			2 => self.get_derivative_2(x),
			_ => {
				if let Some((curr_n, curr_n_func, _)) = &self.nth_derivative {
					if curr_n == &n {
						return curr_n_func.eval(&[x]);
					}
				}
				let new_func = self.function.partial_iter(n);

				self.nth_derivative = Some((
					n,
					new_func.clone(),
					prettyify_function_str(new_func.get_string()),
				));
				new_func.eval(&[x])
			}
		}
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
