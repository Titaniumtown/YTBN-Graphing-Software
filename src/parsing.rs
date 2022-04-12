use exmex::prelude::*;

lazy_static::lazy_static! {
	/// Function returns `f64::NaN` at every x value, which is not displayed.
	static ref EMPTY_FUNCTION: FlatEx<f64> = exmex::parse::<f64>("0/0").unwrap();
}
/// Function that includes f(x), f'(x), f'(x)'s string representation, and
/// f''(x)
#[derive(Clone)]
pub struct BackingFunction {
	/// f(x)
	function: FlatEx<f64>,
	/// f'(x)
	derivative_1: FlatEx<f64>,
	/// Mathematical representation of f'(x)
	derivative_1_str: String,
	/// f''(x)
	derivative_2: FlatEx<f64>,

	nth_derivative: Option<(usize, FlatEx<f64>, String)>,
}

impl BackingFunction {
	/// Create new [`BackingFunction`] instance
	pub fn new(func_str: &str) -> Result<Self, String> {
		let function = match func_str {
			"" => EMPTY_FUNCTION.clone(),
			_ => {
				let parse_result = exmex::parse::<f64>(func_str);

				match &parse_result {
					Err(e) => return Err(e.to_string()),
					Ok(_) => {
						let var_names = parse_result.as_ref().unwrap().var_names().to_vec();

						if var_names != ["x"] {
							let var_names_not_x: Vec<&String> = var_names
								.iter()
								.filter(|ele| ele != &"x")
								.collect::<Vec<&String>>();

							return Err(match var_names_not_x.len() {
								1 => {
									format!("Error: invalid variable: {}", var_names_not_x[0])
								}
								_ => {
									format!("Error: invalid variables: {:?}", var_names_not_x)
								}
							});
						}
					}
				}
				parse_result.unwrap()
			}
		};

		let derivative_1 = function
			.partial(0)
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());

		let derivative_1_str = prettyify_function_str(derivative_1.unparse());

		let derivative_2 = function
			.partial_iter([0, 0].iter())
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());

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
	pub fn get(&self, x: f64) -> f64 { self.function.eval(&[x]).unwrap_or(f64::NAN) }

	/// Calculate f'(x)
	pub fn get_derivative_1(&self, x: f64) -> f64 {
		self.derivative_1.eval(&[x]).unwrap_or(f64::NAN)
	}

	/// Calculate f''(x)
	pub fn get_derivative_2(&self, x: f64) -> f64 {
		self.derivative_2.eval(&[x]).unwrap_or(f64::NAN)
	}

	pub fn get_nth_derivative_str(&self) -> &str { &self.nth_derivative.as_ref().unwrap().2 }

	pub fn get_nth_derivative(&mut self, n: usize, x: f64) -> f64 {
		match n {
			0 => self.get(x),
			1 => self.get_derivative_1(x),
			2 => self.get_derivative_2(x),
			_ => {
				if let Some((curr_n, curr_n_func, _)) = &self.nth_derivative {
					if curr_n == &n {
						return curr_n_func.eval(&[x]).unwrap_or(f64::NAN);
					}
				}
				let new_func = self
					.function
					.partial_iter((1..=n).map(|_| 0).collect::<Vec<usize>>().iter())
					.unwrap_or_else(|_| EMPTY_FUNCTION.clone());

				self.nth_derivative = Some((
					n,
					new_func.clone(),
					prettyify_function_str(new_func.unparse()),
				));
				new_func.eval(&[x]).unwrap_or(f64::NAN)
			}
		}
	}
}

fn prettyify_function_str(func: &str) -> String {
	let new_str = func.to_owned().replace("{x}", "x");

	if &new_str == "0/0" {
		"Undefined".to_owned()
	} else {
		new_str
	}
}

const VALID_VARIABLES: [char; 5] = ['x', 'X', 'e', 'E', 'π'];
const LETTERS: [char; 52] = [
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
	't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
	'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/*
EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed. This is so cursed. But it works, and I hopefully won't ever have to touch it again.
One limitation though, variables with multiple characters like `pi` cannot be multiplied (like `pipipipi` won't result in `pi*pi*pi*pi`). But that's such a niche use case (and that same thing could be done by using exponents) that it doesn't really matter.
In the future I may want to completely rewrite this or implement this natively in exmex.
*/
pub fn process_func_str(function_in: &str) -> String {
	let function = function_in
		.replace("log10(", "log(") // log10 -> log
		.replace("pi", "π") // pi -> π
		.replace("exp", "\u{1fc93}"); // replace 'exp' with this random unicode character because it can't be parsed correctly
	let function_chars: Vec<char> = function.chars().collect();
	let mut output_string: String = String::new();
	for (i, c) in function_chars.iter().enumerate() {
		let mut add_asterisk: bool = false;

		let prev_prev_prev_char = if i > 2 {
			*function_chars.get(i - 3).unwrap()
		} else {
			' '
		};

		let prev_prev_char = if i > 1 {
			*function_chars.get(i - 2).unwrap()
		} else {
			' '
		};

		let prev_char = if i > 0 {
			*function_chars.get(i - 1).unwrap()
		} else {
			' '
		};

		let c_is_number = NUMBERS.contains(c);
		let c_is_letter = LETTERS.contains(c);
		let c_is_variable = VALID_VARIABLES.contains(c);
		let prev_char_is_variable = VALID_VARIABLES.contains(&prev_char);
		let prev_char_is_number = NUMBERS.contains(&prev_char);

		// makes special case for log with base of a 1-2 digit number
		if ((prev_prev_prev_char == 'l')
			&& (prev_prev_char == 'o')
			&& (prev_char == 'g')
			&& c_is_number)
			| ((prev_prev_char == 'c') && (prev_char == 'e') && (*c == 'i'))
		{
			output_string += &c.to_string();
			continue;
		}

		let c_letters_var = c_is_letter | c_is_variable;
		let prev_letters_var = prev_char_is_variable | LETTERS.contains(&prev_char);

		if prev_char == ')' {
			// cases like `)x`, `)2`, and `)(`
			if c_letters_var | c_is_number | (*c == '(') {
				add_asterisk = true;
			}
		} else if *c == '(' {
			// cases like `x(` and `2(`
			if (prev_char_is_variable | prev_char_is_number) && !LETTERS.contains(&prev_prev_char) {
				add_asterisk = true;
			}
		} else if prev_char_is_number {
			// cases like `2x` and `2sin(x)`
			if c_letters_var {
				add_asterisk = true;
			}
		} else if c_is_letter {
			// cases like `e2` and `xx`
			if prev_char_is_number
				| (prev_char_is_variable && c_is_variable)
				| prev_char_is_variable
				| (prev_char == 'π')
			{
				add_asterisk = true;
			}
		} else if (c_is_number | c_letters_var) && prev_letters_var {
			// cases like `x2` and `xx`
			add_asterisk = true;
		}

		// if add_asterisk is true, add the asterisk
		if add_asterisk {
			output_string += "*";
		}

		// push current char to `output_string` (which is eventually returned)
		output_string += &c.to_string();
	}

	output_string
		.replace("log(", "log10(")
		.replace('\u{1fc93}', "exp")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::suggestions::SUPPORTED_FUNCTIONS;
	use std::collections::HashMap;

	/// returns if function with string `func_str` is valid after processing through [`process_func_str`]
	fn func_is_valid(func_str: &str) -> bool {
		BackingFunction::new(&process_func_str(func_str)).is_ok()
	}

	/// Used for testing: passes function to [`process_func_str`] before running [`test_func`]. if `expect_valid` == `true`, it expects no errors to be created.
	fn test_func_helper(func_str: &str, expect_valid: bool) {
		let is_valid = func_is_valid(func_str);
		println!(
			"function: {} (expected: {}, got: {})",
			func_str, expect_valid, is_valid
		);

		assert!(is_valid == expect_valid);
	}

	/// Tests to make sure functions that are expected to succeed, succeed.
	#[test]
	fn test_expected_func_successes() {
		let functions = vec![
			"x^2",
			"2x",
			"E^x",
			"log10(x)",
			"xxxxx", // test variables side-by-side
			"sin(x)",
			"xsin(x)",      // Tests `x{letter}` pattern
			"sin(x)cos(x)", // Tests `){letter}` pattern
			"x/0",          // always returns NaN
			"(x+1)(x-3)",   // tests 2 parentheses in `)(` pattern
			"(2x+1)x",
			"(2x+1)pi",
			"pi(2x+1)",
			"pipipipipipix",
			"e^sin(x)",
			"E^sin(x)",
			"e^x",
		];

		for func_str in functions.iter().cloned() {
			test_func_helper(func_str, true);
		}
	}

	/// Tests to make sure functions that are expected to fail, fail.
	#[test]
	fn test_expected_func_failures() {
		let functions = vec![
			"a",            // Invalid variable
			"l^2",          // Invalid variable
			"log222(x)",    // Invalid function
			"abcdef",       // Invalid variables
			"log10(x",      // unclosed bracket
			"x^a",          // Invalid variable
			"sin(cos(x)))", // extra bracket
			"((())",        // extra opening bracket
			"0/0",
		];

		for func_str in functions.iter().cloned() {
			test_func_helper(func_str, false);
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
			("emax(x)", "e*max(x)"),
			("pisin(x)", "π*sin(x)"),
			("e^sin(x)", "e^sin(x)"),
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
