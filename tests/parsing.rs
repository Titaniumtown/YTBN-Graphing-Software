use parsing::{Hint, SUPPORTED_FUNCTIONS};
use std::collections::HashMap;

#[test]
fn hashmap_gen_test() {
	let data = vec!["time", "text", "test"];
	let expect = vec![
		("t", r#"Hint::Many(&["ime(", "ext(", "est("])"#),
		("ti", r#"Hint::Single("me(")"#),
		("tim", r#"Hint::Single("e(")"#),
		("time", r#"Hint::Single("(")"#),
		("te", r#"Hint::Many(&["xt(", "st("])"#),
		("tex", r#"Hint::Single("t(")"#),
		("text", r#"Hint::Single("(")"#),
		("tes", r#"Hint::Single("t(")"#),
		("test", r#"Hint::Single("(")"#),
	];

	assert_eq!(
		parsing::compile_hashmap(data.iter().map(|e| e.to_string()).collect()),
		expect
			.iter()
			.map(|(a, b)| (a.to_string(), b.to_string()))
			.collect::<Vec<(String, String)>>()
	);
}

/// Returns if function with string `func_str` is valid after processing through [`process_func_str`]
fn func_is_valid(func_str: &str) -> bool {
	parsing::BackingFunction::new(&parsing::process_func_str(func_str)).is_ok()
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
fn test_process_helper(input: &str, expected: &str) {
	assert_eq!(&parsing::process_func_str(input), expected);
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

/// Tests to make sure hints are properly outputed based on input
#[test]
fn hints() {
	let values = HashMap::from([
		("", Hint::Single("x^2")),
		("si", Hint::Many(&["n(", "nh(", "gnum("])),
		("log", Hint::Many(&["2(", "10("])),
		("cos", Hint::Many(&["(", "h("])),
		("sin(", Hint::Single(")")),
		("sqrt", Hint::Single("(")),
		("ln(x)", Hint::None),
		("ln(x)cos", Hint::Many(&["(", "h("])),
		("ln(x)*cos", Hint::Many(&["(", "h("])),
		("sin(cos", Hint::Many(&["(", "h("])),
	]);

	for (key, value) in values {
		println!("{} + {:?}", key, value);
		assert_eq!(parsing::generate_hint(key), &value);
	}
}

#[test]
fn hint_to_string() {
	let values = HashMap::from([
		("x^2", Hint::Single("x^2")),
		(
			r#"["n(", "nh(", "gnum("]"#,
			Hint::Many(&["n(", "nh(", "gnum("]),
		),
		(r#"["n("]"#, Hint::Many(&["n("])),
		("None", Hint::None),
	]);

	for (key, value) in values {
		assert_eq!(value.to_string(), key);
	}
}

#[test]
fn invalid_function() {
	use parsing::SplitType;

	SUPPORTED_FUNCTIONS
		.iter()
		.flat_map(|func1| {
			SUPPORTED_FUNCTIONS
				.iter()
				.map(|func2| func1.to_string() + func2)
				.collect::<Vec<String>>()
		})
		.filter(|func| !SUPPORTED_FUNCTIONS.contains(&func.as_str()))
		.for_each(|key| {
			let split = parsing::split_function(&key, SplitType::Multiplication);

			if split.len() != 1 {
				panic!("failed: {} (len: {}, split: {:?})", key, split.len(), split);
			}

			let generated_hint = parsing::generate_hint(&key);
			if generated_hint.is_none() {
				println!("success: {}", key);
			} else {
				panic!("failed: {} (Hint: '{}')", key, generated_hint);
			}
		});
}

#[test]
fn split_function_multiplication() {
	use parsing::SplitType;

	let values = HashMap::from([
		("cos(x)", vec!["cos(x)"]),
		("cos(", vec!["cos("]),
		("cos(x)sin(x)", vec!["cos(x)", "sin(x)"]),
		("aaaaaaaaaaa", vec!["aaaaaaaaaaa"]),
		("emax(x)", vec!["e", "max(x)"]),
		("x", vec!["x"]),
		("xxx", vec!["x", "x", "x"]),
		("sin(cos(x)x)", vec!["sin(cos(x)", "x)"]),
		("sin(x)*cos(x)", vec!["sin(x)", "cos(x)"]),
		("x*x", vec!["x", "x"]),
		("10*10", vec!["10", "10"]),
		("a1b2c3d4", vec!["a1b2c3d4"]),
		("cos(sin(x)cos(x))", vec!["cos(sin(x)", "cos(x))"]),
	]);

	for (key, value) in values {
		assert_eq!(
			parsing::split_function(key, SplitType::Multiplication),
			value
		);
	}
}

#[test]
fn split_function_terms() {
	use parsing::SplitType;

	let values = HashMap::from([(
		"cos(sin(x)cos(x))",
		vec!["cos(", "sin(", "x)", "cos(", "x))"],
	)]);

	for (key, value) in values {
		assert_eq!(parsing::split_function(key, SplitType::Term), value);
	}
}

#[test]
fn hint_tests() {
	{
		let hint = Hint::None;
		assert!(hint.is_none());
		assert!(!hint.is_some());
		assert!(!hint.is_single());
	}

	{
		let hint = Hint::Single("");
		assert!(!hint.is_none());
		assert!(hint.is_some());
		assert!(hint.is_single());
	}

	{
		let hint = Hint::Many(&[""]);
		assert!(!hint.is_none());
		assert!(hint.is_some());
		assert!(!hint.is_single());
	}
}

#[test]
fn get_last_term() {
	let values = HashMap::from([
		("cos(x)", "x)"),
		("cos(", "cos("),
		("aaaaaaaaaaa", "aaaaaaaaaaa"),
		("x", "x"),
		("xxx", "x"),
		("x*x", "x"),
		("10*10", "10"),
		("sin(cos", "cos"),
		("exp(cos(exp(sin", "sin"),
	]);

	for (key, value) in values {
		assert_eq!(
			parsing::get_last_term(key.chars().collect::<Vec<char>>().as_slice()),
			value
		);
	}
}

#[test]
fn hint_accessor() {
	assert_eq!(Hint::Single("hint").many(), None);
	assert_eq!(Hint::Single("hint").single(), Some(&"hint"));

	assert_eq!(Hint::Many(&["hint", "hint2"]).single(), None);
	assert_eq!(
		Hint::Many(&["hint", "hint2"]).many(),
		Some(&["hint", "hint2"].as_slice())
	);

	assert_eq!(Hint::None.single(), None);
	assert_eq!(Hint::None.many(), None);
}
