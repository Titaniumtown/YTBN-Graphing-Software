use exmex::prelude::*;

lazy_static::lazy_static! {
	/// Function returns `f64::NaN` at every x value, which is not displayed.
	static ref EMPTY_FUNCTION: FlatEx<f64> = exmex::parse::<f64>("0/0").unwrap();
}

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
}

impl BackingFunction {
	/// Create new [`BackingFunction`] instance
	pub fn new(func_str: &str) -> Self {
		let function = match func_str {
			"" => EMPTY_FUNCTION.clone(),
			_ => exmex::parse::<f64>(func_str).unwrap(),
		};

		let derivative_1 = function
			.partial(0)
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());
		let derivative_1_str = derivative_1.unparse().to_owned().replace("{x}", "x");

		let derivative_2 = function
			.partial_iter([0, 0].iter())
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());

		Self {
			function,
			derivative_1,
			derivative_1_str,
			derivative_2,
		}
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
	let function = function_in.replace("log10(", "log(").replace("pi", "π"); // pi -> π and log10 -> log
	let function_chars: Vec<char> = function.chars().collect();
	let mut output_string: String = String::new();
	let mut prev_chars: Vec<char> = Vec::new();
	for c in function_chars {
		let mut add_asterisk: bool = false;
		let prev_chars_len = prev_chars.len();

		let prev_prev_prev_char = if prev_chars_len >= 3 {
			*prev_chars.get(prev_chars_len - 3).unwrap()
		} else {
			' '
		};

		let prev_prev_char = if prev_chars_len >= 2 {
			*prev_chars.get(prev_chars_len - 2).unwrap()
		} else {
			' '
		};

		let prev_char = if prev_chars_len >= 1 {
			*prev_chars.get(prev_chars_len - 1).unwrap()
		} else {
			' '
		};

		let c_is_number = NUMBERS.contains(&c);
		let c_is_letter = LETTERS.contains(&c);
		let c_is_variable = VALID_VARIABLES.contains(&c);
		let prev_char_is_variable = VALID_VARIABLES.contains(&prev_char);
		let prev_char_is_number = NUMBERS.contains(&prev_char);

		// makes special case for log with base of a 1-2 digit number
		if (prev_prev_prev_char == 'l')
			&& (prev_prev_char == 'o')
			&& (prev_char == 'g')
			&& c_is_number
		{
			prev_chars.push(c);
			output_string += &c.to_string();
			continue;
		}

		let c_letters_var = c_is_letter | c_is_variable;
		let prev_letters_var = prev_char_is_variable | LETTERS.contains(&prev_char);

		if prev_char == ')' {
			// cases like `)x`, `)2`, and `)(`
			if c_letters_var | c_is_number | (c == '(') {
				add_asterisk = true;
			}
		} else if c == '(' {
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

		// puch current char to vector of previously parsed chars
		prev_chars.push(c);

		// push current char to `output_string` (which is eventually returned)
		output_string += &c.to_string();
	}

	output_string.replace("log(", "log10(")
}

/// Tests function to make sure it's able to be parsed. Returns the string of
/// the Error produced, or an empty string if it runs successfully.
pub fn test_func(function_string: &str) -> Option<String> {
	if function_string.is_empty() {
		return None;
	}

	let parse_result = exmex::parse::<f64>(function_string);

	match parse_result {
		Err(e) => Some(e.to_string()),
		Ok(_) => {
			let var_names = parse_result.unwrap().var_names().to_vec();

			if var_names != ["x"] {
				let var_names_not_x: Vec<String> = var_names
					.iter()
					.filter(|ele| *ele != "x")
					.cloned()
					.collect::<Vec<String>>();

				return match var_names_not_x.len() {
					1 => {
						let var_name = &var_names_not_x[0];
						if var_name == "e" {
							Some(String::from(
								"If trying to use Euler's number, please use an uppercase E",
							))
						} else {
							Some(format!("Error: invalid variable: {}", var_name))
						}
					}
					_ => Some(format!("Error: invalid variables: {:?}", var_names_not_x)),
				};
			}

			None
		}
	}
}

pub fn generate_hint(input: &str) -> String {
	if input.is_empty() {
		return "x^2".to_owned();
	}

	let chars: Vec<char> = input.chars().collect();

	let open_parens = chars.iter().filter(|c| **c == '(').count();
	let closed_parents = chars.iter().filter(|c| **c == ')').count();

	if open_parens > closed_parents {
		return ")".to_owned();
	}

	let len = chars.len();

	if chars.len() >= 5 {
		let result_two = match (chars[len - 5].to_string()
			+ &chars[len - 4].to_string()
			+ &chars[len - 3].to_string()
			+ &chars[len - 2].to_string()
			+ &chars[len - 1].to_string())
			.as_str()
		{
			"round" => Some("("),
			"fract" => Some("("),
			"trunc" => Some("("),
			"floor" => Some("("),

			_ => None,
		};

		if let Some(output) = result_two {
			return output.to_owned();
		}
	}

	if chars.len() >= 4 {
		let result_two = match (chars[len - 4].to_string()
			+ &chars[len - 3].to_string()
			+ &chars[len - 2].to_string()
			+ &chars[len - 1].to_string())
			.as_str()
		{
			"asin" => Some("("),
			"acos" => Some("("),
			"atan" => Some("("),

			"sinh" => Some("("),
			"cosh" => Some("("),
			"tanh" => Some("("),

			"ceil" => Some("("),
			"roun" => Some("d("),
			"sqrt" => Some("("),
			"cbrt" => Some("("),
			"floo" => Some("r("),

			"frac" => Some("t("),

			_ => None,
		};

		if let Some(output) = result_two {
			return output.to_owned();
		}
	}

	if chars.len() >= 3 {
		let result_two = match (chars[len - 3].to_string()
			+ &chars[len - 2].to_string()
			+ &chars[len - 1].to_string())
			.as_str()
		{
			"flo" => Some("or("),
			"log" => Some("("),
			"abs" => Some("("),

			"sin" => Some("("),
			"cos" => Some("("),
			"tan" => Some("("),

			"asi" => Some("n("),
			"aco" => Some("s("),
			"ata" => Some("n("),
			"exp" => Some("("),
			"fra" => Some("ct("),
			"cbr" => Some("t("),
			"cei" => Some("l("),

			_ => None,
		};

		if let Some(output) = result_two {
			return output.to_owned();
		}
	}

	if chars.len() >= 2 {
		let result_two = match (chars[len - 2].to_string() + &chars[len - 1].to_string()).as_str() {
			"lo" => Some("g("),
			"si" => Some("n("),
			"ab" => Some("s("),
			"co" => Some("s("),
			"ta" => Some("n("),
			"as" => Some("in("),
			"ac" => Some("os("),
			"at" => Some("an("),
			"ln" => Some("("),
			"fl" => Some("oor("),
			"sq" => Some("rt("),
			"ex" => Some("p("),
			"ce" => Some("il("),

			_ => None,
		};

		if let Some(output) = result_two {
			return output.to_owned();
		}
	}

	String::new()
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	/// returns if function with string `func_str` is valid after processing
	/// through [`process_func_str`]
	fn func_is_valid(func_str: &str) -> bool { test_func(&process_func_str(func_str)).is_none() }

	/// Used for testing: passes function to [`process_func_str`] before running
	/// [`test_func`]. if `expect_valid` == `true`, it expects no errors to be
	/// created.
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
			"xsin(x)",
			"sin(x)cos(x)",
			"x/0",        // always returns NaN
			"(x+1)(x-3)", // tests 2 parentheses in `)(` pattern
			"(2x+1)x",
			"(2x+1)pi",
			"pi(2x+1)",
			"pipipipipipix",
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
			"((())",
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
			("x2", "x*2"),
			("x(1+3)", "x*(1+3)"),
			("(1+3)x", "(1+3)*x"),
			("sin(x)", "sin(x)"),
			("2sin(x)", "2*sin(x)"),
			("max(x)", "max(x)"),
			("2e^x", "2*e^x"),
			("2max(x)", "2*max(x)"),
			("cos(sin(x))", "cos(sin(x))"),
			("x^(1+2x)", "x^(1+2*x)"),
			("(x+2)x(1+3)", "(x+2)*x*(1+3)"),
			("(x+2)(1+3)", "(x+2)*(1+3)"),
			("xxx", "x*x*x"),
			("eee", "e*e*e"),
			("pi(x+2)", "π*(x+2)"),
			("(x)pi", "(x)*π"),
			("2e", "2*e"),
			("2log10(x)", "2*log10(x)"),
			("2log(x)", "2*log10(x)"),
			("x!", "x!"),
			("pipipipipipi", "π*π*π*π*π*π"),
			("10pi", "10*π"),
			("pi10", "π*10"),
			("emax(x)", "e*max(x)"),
			("pisin(x)", "π*sin(x)"),
		]);

		for (key, value) in values {
			test_process_helper(key, value);
		}
	}

	/// Tests to make sure hints are properly outputed based on input
	#[test]
	fn hint_test() {
		let values = HashMap::from([
			("", "x^2"),
			("sin(x", ")"),
			("sin(x)", ""),
			("x^x", ""),
			("(x+1)(x-1", ")"),
			("lo", "g("),
			("log", "("),
			("asi", "n("),
			("si", "n("),
			("asin", "("),
			("fl", "oor("),
			("ata", "n("),
			("at", "an("),
			("roun", "d("),
			("floo", "r("),
			("flo", "or("),
		]);

		for (key, value) in values {
			assert_eq!(generate_hint(key), value.to_owned());
		}
	}
}
