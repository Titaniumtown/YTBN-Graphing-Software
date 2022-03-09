use exmex::prelude::*;

#[derive(Clone)]
pub struct BackingFunction {
    function: FlatEx<f64>,
    derivative_1: FlatEx<f64>,
    // derivative_2: FlatEx<f64>,
}

impl BackingFunction {
    pub fn new(func_str: &str) -> Self {
        let function = exmex::parse::<f64>(func_str).unwrap();
        let derivative_1 = function.partial(0).unwrap_or_else(|_| function.clone());
        // let derivative_2 = function.partial(0).unwrap_or(derivative_1.clone());

        Self {
            function,
            derivative_1,
            // derivative_2,
        }
    }

    pub fn get_derivative_str(&self) -> String {
        String::from(self.derivative_1.unparse()).replace("{x}", "x")
    }

    pub fn get(&self, x: f64) -> f64 { self.function.eval(&[x]).unwrap_or(f64::NAN) }

    pub fn derivative(&self, x: f64) -> f64 { self.derivative_1.eval(&[x]).unwrap_or(f64::NAN) }
}

lazy_static::lazy_static! {
    static ref VALID_VARIABLES: Vec<char> = "xXEπ".chars().collect();
    static ref LETTERS: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();
    static ref NUMBERS: Vec<char> = "0123456789".chars().collect();
}

/*
EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed. This is so cursed. But it works, and I hopefully won't ever have to touch it again.
One limitation though, variables with multiple characters like `pi` cannot be multiplied (like `pipipipi` won't result in `pi*pi*pi*pi`). But that's such a niche use case (and that same thing could be done by using exponents) that it doesn't really matter.
In the future I may want to completely rewrite this or implement this natively into mevel-rs (which would probably be good to do)
*/
pub fn add_asterisks(function_in: String) -> String {
    let function = function_in.replace("log10(", "log(").replace("pi", "π"); // pi -> π and log10 -> log
    let function_chars: Vec<char> = function.chars().collect();
    let mut output_string: String = String::new();
    let mut prev_chars: Vec<char> = Vec::new();
    for c in function_chars {
        let mut add_asterisk: bool = false;
        let prev_chars_len = prev_chars.len();

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

    output_string
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
                    1 => Some(format!("Error: invalid variable: {}", var_names_not_x[0])),
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
    test_func(&add_asterisks(function_string.to_string()))
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

// Tests to make sure my cursed function works as intended
#[test]
fn asterisk_test() {
    assert_eq!(&add_asterisks("2x".to_string()), "2*x");
    assert_eq!(&add_asterisks("x2".to_string()), "x*2");
    assert_eq!(&add_asterisks("x(1+3)".to_string()), "x*(1+3)");
    assert_eq!(&add_asterisks("(1+3)x".to_string()), "(1+3)*x");
    assert_eq!(&add_asterisks("sin(x)".to_string()), "sin(x)");
    assert_eq!(&add_asterisks("2sin(x)".to_string()), "2*sin(x)");
    assert_eq!(&add_asterisks("max(x)".to_string()), "max(x)");
    assert_eq!(&add_asterisks("2e^x".to_string()), "2*e^x");
    assert_eq!(&add_asterisks("2max(x)".to_string()), "2*max(x)");
    assert_eq!(&add_asterisks("cos(sin(x))".to_string()), "cos(sin(x))");
    assert_eq!(&add_asterisks("x^(1+2x)".to_string()), "x^(1+2*x)");
    assert_eq!(&add_asterisks("(x+2)x(1+3)".to_string()), "(x+2)*x*(1+3)");
    assert_eq!(&add_asterisks("(x+2)(1+3)".to_string()), "(x+2)*(1+3)");
    assert_eq!(&add_asterisks("xxx".to_string()), "x*x*x");
    assert_eq!(&add_asterisks("eee".to_string()), "e*e*e");
    assert_eq!(&add_asterisks("pi(x+2)".to_string()), "π*(x+2)");
    assert_eq!(&add_asterisks("(x)pi".to_string()), "(x)*π");
    assert_eq!(&add_asterisks("2e".to_string()), "2*e");
    assert_eq!(&add_asterisks("2log10(x)".to_string()), "2*log(x)");
    assert_eq!(&add_asterisks("2log(x)".to_string()), "2*log(x)");
    assert_eq!(&add_asterisks("x!".to_string()), "x!");
    assert_eq!(&add_asterisks("pipipipipipi".to_string()), "π*π*π*π*π*π");
    assert_eq!(&add_asterisks("10pi".to_string()), "10*π");
    assert_eq!(&add_asterisks("pi10".to_string()), "π*10");

    // Need to fix these checks, maybe I need to rewrite the whole asterisk adding system... (or just implement these changes into meval-rs, idk)
    // assert_eq!(&add_asterisks("emax(x)".to_string()), "e*max(x)");
    // assert_eq!(&add_asterisks("pisin(x)".to_string()), "pi*sin(x)");
}
