use crate::parsing::is_variable;

pub fn split_function(input: &str, split: SplitType) -> Vec<String> {
	split_function_chars(
		&input
			.replace("pi", "π") // replace "pi" text with pi symbol
			.replace("**", "^") // support alternate manner of expressing exponents
			.replace("exp", "\u{1fc93}") // stop-gap solution to fix the `exp` function
			.chars()
			.collect::<Vec<char>>(),
		split,
	)
	.iter()
	.map(|x| x.replace('\u{1fc93}', "exp")) // Convert back to `exp` text
	.collect::<Vec<String>>()
}

// Specifies how to split a function
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SplitType {
	Multiplication,
	Term,
}

/// Used to store info about a character
struct BoolSlice {
	closing_parens: bool,
	open_parens: bool,
	number: bool,
	letter: bool,
	variable: bool,
	masked_num: bool,
	masked_var: bool,
}

impl BoolSlice {
	const fn from_char(c: &char, prev_masked_num: bool, prev_masked_var: bool) -> Self {
		let isnumber = c.is_ascii_digit();
		let isvariable = is_variable(c);
		Self {
			closing_parens: *c == ')',
			open_parens: *c == '(',
			number: isnumber,
			letter: c.is_ascii_alphabetic(),
			variable: isvariable,
			masked_num: match isnumber {
				true => prev_masked_num,
				false => false,
			},
			masked_var: match isvariable {
				true => prev_masked_var,
				false => false,
			},
		}
	}

	const fn is_unmasked_variable(&self) -> bool { self.variable && !self.masked_var }

	const fn is_unmasked_number(&self) -> bool { self.number && !self.masked_num }

	const fn calculate_mask(&mut self, other: &BoolSlice) {
		if other.masked_num && self.number {
			// If previous char was a masked number, and current char is a number, mask current char's variable status
			self.masked_num = true;
		} else if other.masked_var && self.variable {
			// If previous char was a masked variable, and current char is a variable, mask current char's variable status
			self.masked_var = true;
		} else if other.letter && !other.is_unmasked_variable() {
			self.masked_num = self.number;
			self.masked_var = self.variable;
		}
	}

	const fn splitable(&self, c: &char, other: &BoolSlice, split: &SplitType) -> bool {
		if (*c == '*') | (matches!(split, &SplitType::Term) && other.open_parens) {
			true
		} else if other.closing_parens {
			// Cases like `)x`, `)2`, and `)(`
			return (*c == '(')
				| (self.letter && !self.is_unmasked_variable())
				| self.is_unmasked_variable()
				| self.is_unmasked_number();
		} else if *c == '(' {
			// Cases like `x(` and `2(`
			return (other.is_unmasked_variable() | other.is_unmasked_number()) && !other.letter;
		} else if other.is_unmasked_number() {
			// Cases like `2x` and `2sin(x)`
			return self.is_unmasked_variable() | self.letter;
		} else if self.is_unmasked_variable() | self.letter {
			// Cases like `e2` and `xx`
			return other.is_unmasked_number()
				| (other.is_unmasked_variable() && self.is_unmasked_variable())
				| other.is_unmasked_variable();
		} else if (self.is_unmasked_number() | self.letter | self.is_unmasked_variable())
			&& (other.is_unmasked_number() | other.letter)
		{
			return true;
		} else {
			return self.is_unmasked_number() && other.is_unmasked_variable();
		}
	}
}

// Splits a function (which is represented as an array of characters) based off of the value of SplitType
pub fn split_function_chars(chars: &[char], split: SplitType) -> Vec<String> {
	// Catch some basic cases
	match chars.len() {
		0 => return Vec::new(),
		1 => return vec![chars[0].to_string()],
		_ => {}
	}

	// Resulting split-up data
	let mut data: Vec<String> = std::vec::from_elem(chars[0].to_string(), 1);

	// Setup first char here
	let mut prev_char: BoolSlice = BoolSlice::from_char(&chars[0], false, false);

	let mut last = unsafe { data.last_mut().unwrap_unchecked() };

	// Iterate through all chars excluding the first one
	for c in chars.iter().skip(1) {
		// Set data about current character
		let mut curr_c = BoolSlice::from_char(c, prev_char.masked_num, prev_char.masked_var);

		curr_c.calculate_mask(&prev_char);

		// Append split
		if curr_c.splitable(c, &prev_char, &split) {
			// create new buffer
			data.push(String::new());
			last = unsafe { data.last_mut().unwrap_unchecked() };
		}

		// Exclude asterisks
		if c != &'*' {
			last.push(*c);
		}

		// Move current character data to `prev_char`
		prev_char = curr_c;
	}

	data
}

#[cfg(test)]
fn assert_test(input: &str, expected: &[&str], split: SplitType) {
	let output = split_function(input, split);
	let expected_owned = expected
		.iter()
		.map(|&x| x.to_owned())
		.collect::<Vec<String>>();
	if output != expected_owned {
		panic!(
			"split type: {:?} of {} resulted in {:?} not {:?}",
			split, input, output, expected
		);
	}
}

#[test]
fn split_function_test() {
	assert_test(
		"sin(x)cos(x)",
		&["sin(x)", "cos(x)"],
		SplitType::Multiplication,
	);

	assert_test(
		"tanh(cos(x)xx)cos(x)",
		&["tanh(cos(x)", "x", "x)", "cos(x)"],
		SplitType::Multiplication,
	);

	assert_test(
		"tanh(sin(cos(x)xsin(x)))",
		&["tanh(sin(cos(x)", "x", "sin(x)))"],
		SplitType::Multiplication,
	);

	// Some test cases from https://github.com/GraphiteEditor/Graphite/blob/2515620a77478e57c255cd7d97c13cc7065dd99d/frontend/wasm/src/editor_api.rs#L829-L840
	assert_test("2pi", &["2", "π"], SplitType::Multiplication);
	assert_test("sin(2pi)", &["sin(2", "π)"], SplitType::Multiplication);
	assert_test("2sin(pi)", &["2", "sin(π)"], SplitType::Multiplication);
	assert_test(
		"2sin(3(4 + 5))",
		&["2", "sin(3", "(4 + 5))"],
		SplitType::Multiplication,
	);
	assert_test("3abs(-4)", &["3", "abs(-4)"], SplitType::Multiplication);
	assert_test("-1(4)", &["-1", "(4)"], SplitType::Multiplication);
	assert_test("(-1)4", &["(-1)", "4"], SplitType::Multiplication);
	assert_test(
		"(((-1)))(4)",
		&["(((-1)))", "(4)"],
		SplitType::Multiplication,
	);
	assert_test(
		"2sin(π) + 2cos(tau)",
		&["2", "sin(π) + 2", "cos(tau)"],
		SplitType::Multiplication,
	);
}
