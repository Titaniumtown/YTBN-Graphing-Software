use exmex::prelude::*;

lazy_static::lazy_static! {
	static ref EMPTY_FUNCTION: FlatEx<f64> = exmex::parse::<f64>("0/0").unwrap();
}

#[derive(Clone)]
pub struct BackingFunction {
	function: FlatEx<f64>,
	derivative_1: FlatEx<f64>,
	derivative_2: FlatEx<f64>,
}

impl BackingFunction {
	pub fn new(func_str: &str) -> Self {
		let function = exmex::parse::<f64>(func_str).unwrap();
		let derivative_1 = function
			.partial(0)
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());
		let derivative_2 = function
			.partial_iter([0, 0].iter())
			.unwrap_or_else(|_| EMPTY_FUNCTION.clone());

		Self {
			function,
			derivative_1,
			derivative_2,
		}
	}

	pub fn get_derivative_str(&self) -> String {
		String::from(self.derivative_1.unparse()).replace("{x}", "x")
	}

	pub fn get(&self, x: f64) -> f64 { self.function.eval(&[x]).unwrap_or(f64::NAN) }

	pub fn get_derivative_1(&self, x: f64) -> f64 {
		self.derivative_1.eval(&[x]).unwrap_or(f64::NAN)
	}

	pub fn get_derivative_2(&self, x: f64) -> f64 {
		self.derivative_2.eval(&[x]).unwrap_or(f64::NAN)
	}
}

lazy_static::lazy_static! {
	static ref VALID_VARIABLES: Vec<char> = "xXeEπ".chars().collect();
	static ref LETTERS: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
		.chars()
		.collect();
	static ref NUMBERS: Vec<char> = "0123456789".chars().collect();
}

/*
EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed. This is so cursed. But it works, and I hopefully won't ever have to touch it again.
One limitation though, variables with multiple characters like `pi` cannot be multiplied (like `pipipipi` won't result in `pi*pi*pi*pi`). But that's such a niche use case (and that same thing could be done by using exponents) that it doesn't really matter.
In the future I may want to completely rewrite this or implement this natively in exmex.
*/
pub fn process_func_str(function_in: String) -> String {
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

		if (prev_prev_prev_char == 'l')
			&& (prev_prev_char == 'o')
			&& (prev_char == 'g')
			&& (NUMBERS.contains(&c))
		{
			prev_chars.push(c);
			output_string += &c.to_string();
			continue;
		}

		let c_letters_var = LETTERS.contains(&c) | VALID_VARIABLES.contains(&c);
		let prev_letters_var = VALID_VARIABLES.contains(&prev_char) | LETTERS.contains(&prev_char);

		if prev_char == ')' {
			if (c == '(') | NUMBERS.contains(&c) | c_letters_var {
				add_asterisk = true;
			}
		} else if c == '(' {
			if (VALID_VARIABLES.contains(&prev_char)
				| (')' == prev_char)
				| NUMBERS.contains(&prev_char))
				&& !LETTERS.contains(&prev_prev_char)
			{
				add_asterisk = true;
			}
		} else if NUMBERS.contains(&prev_char) {
			if (c == '(') | c_letters_var {
				add_asterisk = true;
			}
		} else if LETTERS.contains(&c) {
			if NUMBERS.contains(&prev_char)
				| (VALID_VARIABLES.contains(&prev_char) && VALID_VARIABLES.contains(&c))
			{
				add_asterisk = true;
			}
		} else if (NUMBERS.contains(&c) | c_letters_var) && prev_letters_var {
			add_asterisk = true;
		}

		if add_asterisk {
			output_string += "*";
		}

		prev_chars.push(c);
		output_string += &c.to_string();
	}

	output_string.replace("log(", "log10(")
}

// Tests function to make sure it's able to be parsed. Returns the string of the Error produced, or an empty string if it runs successfully.
pub fn test_func(function_string: &str) -> Option<String> {
	let parse_result = exmex::parse::<f64>(function_string);

	match parse_result {
		Err(e) => Some(e.to_string()),
		Ok(_) => {
			let var_names = parse_result.unwrap().var_names().to_vec();

			if var_names != ["x"] {
				let var_names_not_x: Vec<String> = var_names
					.iter()
					.filter(|ele| *ele != &"x".to_owned())
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

// Used for testing: passes function to `add_asterisks` before running `test_func`
#[cfg(test)]
fn test_func_helper(function_string: &str) -> Option<String> {
	test_func(&process_func_str(function_string.to_string()))
}

#[test]
fn test_func_test() {
	// These shouldn't fail
	assert!(test_func_helper("x^2").is_none());
	assert!(test_func_helper("2x").is_none());
	// assert!(test_func_helper("e^x").is_none()); // need to fix!!! PR to exmex
	assert!(test_func_helper("E^x").is_none());
	assert!(test_func_helper("log10(x)").is_none());
	assert!(test_func_helper("xxxxx").is_none());

	// Expect these to fail
	assert!(test_func_helper("a").is_some());
	assert!(test_func_helper("l^2").is_some());
	assert!(test_func_helper("log222(x)").is_some());
	assert!(test_func_helper("abcdef").is_some());
}

#[cfg(test)]
fn test_process_helper(input: &str, expected: &str) {
	assert_eq!(&process_func_str(input.to_string()), expected);
}

// Tests to make sure my cursed function works as intended
#[test]
fn func_process_test() {
	test_process_helper("2x", "2*x");
	test_process_helper("x2", "x*2");
	test_process_helper("x(1+3)", "x*(1+3)");
	test_process_helper("(1+3)x", "(1+3)*x");
	test_process_helper("sin(x)", "sin(x)");
	test_process_helper("2sin(x)", "2*sin(x)");
	test_process_helper("max(x)", "max(x)");
	test_process_helper("2e^x", "2*e^x");
	test_process_helper("2max(x)", "2*max(x)");
	test_process_helper("cos(sin(x))", "cos(sin(x))");
	test_process_helper("x^(1+2x)", "x^(1+2*x)");
	test_process_helper("(x+2)x(1+3)", "(x+2)*x*(1+3)");
	test_process_helper("(x+2)(1+3)", "(x+2)*(1+3)");
	test_process_helper("xxx", "x*x*x");
	test_process_helper("eee", "e*e*e");
	test_process_helper("pi(x+2)", "π*(x+2)");
	test_process_helper("(x)pi", "(x)*π");
	test_process_helper("2e", "2*e");
	test_process_helper("2log10(x)", "2*log10(x)");
	test_process_helper("2log(x)", "2*log10(x)");
	test_process_helper("x!", "x!");
	test_process_helper("pipipipipipi", "π*π*π*π*π*π");
	test_process_helper("10pi", "10*π");
	test_process_helper("pi10", "π*10");

	// Need to fix these checks, maybe I need to rewrite the whole asterisk adding system... (or just implement these changes into meval-rs, idk)
	// test_process_helper("emax(x)", "e*max(x)");
	// test_process_helper("pisin(x)", "pi*sin(x)");
}
