use exmex::prelude::*;

#[derive(Clone)]
pub(crate) struct FlatExWrapper {
	func: Option<FlatEx<f64>>,
}

impl FlatExWrapper {
	const EMPTY: FlatExWrapper = FlatExWrapper { func: None };

	const fn new(f: FlatEx<f64>) -> Self { Self { func: Some(f) } }

	const fn is_none(&self) -> bool { self.func.is_none() }

	fn eval(&self, x: &[f64]) -> f64 {
		self.func
			.as_ref()
			.map(|f| f.eval(x).unwrap_or(f64::NAN))
			.unwrap_or(f64::NAN)
	}

	fn partial(&self, x: usize) -> Self {
		self.func
			.as_ref()
			.map(|f| f.partial(x).map(|a| Self::new(a)).unwrap_or(Self::EMPTY))
			.unwrap_or(Self::EMPTY)
	}

	fn get_string(&self) -> &str { self.func.as_ref().map(|f| f.unparse()).unwrap_or("") }

	fn partial_iter(&self, x: &[usize]) -> Self {
		self.func
			.as_ref()
			.map(|f| {
				f.partial_iter(x.iter())
					.map(|a| Self::new(a))
					.unwrap_or(Self::EMPTY)
			})
			.unwrap_or(Self::EMPTY)
	}
}

impl const Default for FlatExWrapper {
	fn default() -> FlatExWrapper { FlatExWrapper::EMPTY }
}

/// Function that includes f(x), f'(x), f'(x)'s string representation, and f''(x)
#[derive(Clone)]
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

	pub fn is_none(&self) -> bool { self.function.is_none() }

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
	pub fn get_nth_derivative_str(&self) -> &str { &self.nth_derivative.as_ref().unwrap().2 }

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
				let new_func = self
					.function
					.partial_iter((1..=n).map(|_| 0).collect::<Vec<usize>>().as_slice());

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

	crate::suggestions::split_function(&function_in).join("*")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::suggestions::SUPPORTED_FUNCTIONS;
	use std::collections::HashMap;

	/// Returns if function with string `func_str` is valid after processing through [`process_func_str`]
	fn func_is_valid(func_str: &str) -> bool {
		BackingFunction::new(&process_func_str(func_str)).is_ok()
	}

	/// Used for testing: passes function to [`process_func_str`] before running [`test_func`]. if `expect_valid` == `true`, it expects no errors to be created.
	fn test_func_helper(func_str: &str, expect_valid: bool) {
		let is_valid = func_is_valid(func_str);
		let string = format!(
			"function: {} (expected: {}, got: {})",
			func_str, expect_valid, is_valid
		);

		if is_valid == expect_valid {
			println!("{}", string);
		} else {
			panic!("{}", string);
		}
	}

	/// Tests to make sure functions that are expected to succeed, succeed.
	#[test]
	fn test_expected() {
		let values = HashMap::from([
			("", true),
			("x^2", true),
			("2x", true),
			("E^x", true),
			("log10(x)", true),
			("xxxxx", true),
			("sin(x)", true),
			("xsin(x)", true),
			("sin(x)cos(x)", true),
			("x/0", true),
			("(x+1)(x-3)", true),
			("cos(xsin(x)x)", true),
			("(2x+1)x", true),
			("(2x+1)pi", true),
			("pi(2x+1)", true),
			("pipipipipipix", true),
			("e^sin(x)", true),
			("E^sin(x)", true),
			("e^x", true),
			("x**2", true),
			("a", false),
			("log222(x)", false),
			("abcdef", false),
			("log10(x", false),
			("x^a", false),
			("sin(cos(x)))", false),
			("0/0", false),
		]);

		for (key, value) in values {
			test_func_helper(key, value);
		}
	}

	/// Helps with tests of [`process_func_str`]
	#[cfg(test)]
	fn test_process_helper(input: &str, expected: &str) {
		assert_eq!(&process_func_str(input), expected);
	}

	/// Tests to make sure my cursed function works as intended
	#[test]
	fn func_process_test() {
		let values = HashMap::from([
			("2x", "2*x"),
			(")(", ")*("),
			("(2", "(2"),
			("log10(x)", "log10(x)"),
			("log2(x)", "log2(x)"),
			("pipipipipipi", "π*π*π*π*π*π"),
			("10pi", "10*π"),
			("pi10", "π*10"),
			("10pi10", "10*π*10"),
			("emax(x)", "e*max(x)"),
			("pisin(x)", "π*sin(x)"),
			("e^sin(x)", "e^sin(x)"),
			("x**2", "x^2"),
			("(x+1)(x-3)", "(x+1)*(x-3)"),
		]);

		for (key, value) in values {
			test_process_helper(key, value);
		}

		for func in SUPPORTED_FUNCTIONS.iter() {
			let func_new = format!("{}(x)", func);
			test_process_helper(&func_new, &func_new);
		}
	}
}
