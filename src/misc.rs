use meval::Expr;

/*
EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed. This is so cursed. But it works, and I hopefully won't ever have to touch it again.
One limitation though, variables with multiple characters like `pi` cannot be multiplied (like `pipipipi` won't result in `pi*pi*pi*pi`). But that's such a niche use case (and that same thing could be done by using exponents) that it doesn't really matter.
In the future I may want to completely rewrite this or implement this natively into mevel-rs (which would probably be good to do)
*/
pub fn add_asterisks(function_in: String) -> String {
    let function = function_in.replace("log10(", "log(").replace("pi", "π"); // pi -> π and log10 -> log
    let valid_variables: Vec<char> = "xeπ".chars().collect();
    let letters: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();
    let numbers: Vec<char> = "0123456789".chars().collect();
    let function_chars: Vec<char> = function.chars().collect();
    let mut output_string: String = String::new();
    let mut prev_chars: Vec<char> = Vec::new();
    for c in function_chars {
        let mut add_asterisk: bool = false;
        let prev_chars_len = prev_chars.len();

        let prev_prev_char = if prev_chars_len >= 2 {
            match prev_chars.get(prev_chars_len - 2) {
                Some(x) => *x,
                None => panic!(),
            }
        } else {
            ' '
        };

        let prev_char = if prev_chars_len >= 1 {
            match prev_chars.get(prev_chars_len - 1) {
                Some(x) => *x,
                None => panic!(),
            }
        } else {
            ' '
        };

        let c_letters_var = letters.contains(&c) | valid_variables.contains(&c);
        let prev_letters_var = valid_variables.contains(&prev_char) | letters.contains(&prev_char);

        if prev_char == ')' {
            if (c == '(') | numbers.contains(&c) | c_letters_var {
                add_asterisk = true;
            }
        } else if c == '(' {
            if (valid_variables.contains(&prev_char)
                | (')' == prev_char)
                | numbers.contains(&prev_char))
                && !letters.contains(&prev_prev_char)
            {
                add_asterisk = true;
            }
        } else if numbers.contains(&prev_char) {
            if (c == '(') | c_letters_var {
                add_asterisk = true;
            }
        } else if letters.contains(&c) {
            if numbers.contains(&prev_char) | (valid_variables.contains(&prev_char) && valid_variables.contains(&c)) {
                add_asterisk = true;
            }
        } else if (numbers.contains(&c) | c_letters_var) && prev_letters_var {
            add_asterisk = true;
        }

        if add_asterisk {
            output_string += "*";
        }

        prev_chars.push(c);
        output_string += &c.to_string();
    }

    output_string.replace('π', "pi") // π -> pi
}

// Tests function to make sure it's able to be parsed. Returns the string of the Error produced, or an empty string if it runs successfully.
pub fn test_func(function_string: String) -> String {
    // Factorials do not work, and it would be really difficult to make them work
    if function_string.contains('!') {
        return "Factorials are unsupported".to_string();
    }

    let new_func_str: String = add_asterisks(function_string);
    let expr_result = new_func_str.parse();
    let expr_error = match &expr_result {
        Ok(_) => "".to_string(),
        Err(error) => format!("{}", error),
    };
    if !expr_error.is_empty() {
        return expr_error;
    }

    let expr: Expr = expr_result.unwrap();
    let func_result = expr.bind("x");
    let func_error = match &func_result {
        Ok(_) => "".to_string(),
        Err(error) => format!("{}", error),
    };
    if !func_error.is_empty() {
        return func_error;
    }

    "".to_string()
}

// Rounds f64 to specific number of digits
pub fn digits_precision(x: f64, digits: usize) -> f64 {
    let large_number: f64 = 10.0_f64.powf(digits as f64);
    (x * large_number).round() / large_number
}

pub struct Function {
    function: Box<dyn Fn(f64) -> f64>,
    func_str: String,
}

impl Function {
    pub fn from_string(func_str: String) -> Self {
        let expr: Expr = func_str.parse().unwrap();
        let func = expr.bind("x").unwrap();
        Self {
            function: Box::new(func),
            func_str,
        }
    }

    #[inline]
    pub fn run(&self, x: f64) -> f64 { (self.function)(x) }

    pub fn str_compare(&self, other_string: String) -> bool { self.func_str == other_string }

    #[allow(dead_code)]
    pub fn get_string(&self) -> String { self.func_str.clone() }
}

pub struct Cache<T> {
    backing_data: Option<T>,
}

impl<T> Cache<T> {
    #[allow(dead_code)]
    #[inline]
    pub fn new(backing_data: T) -> Self {
        Self {
            backing_data: Some(backing_data),
        }
    }

    #[inline]
    pub fn new_empty() -> Self { Self { backing_data: None } }

    #[inline]
    pub fn get(&self) -> &T {
        match &self.backing_data {
            Some(x) => x,
            None => panic!("self.backing_data is None"),
        }
    }

    #[inline]
    pub fn set(&mut self, data: T) { self.backing_data = Some(data); }

    #[inline]
    pub fn invalidate(&mut self) { self.backing_data = None; }

    #[inline]
    pub fn is_valid(&self) -> bool {
        match &self.backing_data {
            Some(_) => true,
            None => false,
        }
    }
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
    assert_eq!(&add_asterisks("pi(x+2)".to_string()), "pi*(x+2)");
    assert_eq!(&add_asterisks("(x)pi".to_string()), "(x)*pi");
    assert_eq!(&add_asterisks("2e".to_string()), "2*e");
    assert_eq!(&add_asterisks("2log10(x)".to_string()), "2*log(x)");
    assert_eq!(&add_asterisks("2log(x)".to_string()), "2*log(x)");
    assert_eq!(&add_asterisks("x!".to_string()), "x!");
    assert_eq!(
        &add_asterisks("pipipipipipi".to_string()),
        "pi*pi*pi*pi*pi*pi"
    );
    assert_eq!(&add_asterisks("10pi".to_string()), "10*pi");
    assert_eq!(&add_asterisks("pi10".to_string()), "pi*10");

    // Need to fix these checks, maybe I need to rewrite the whole asterisk adding system... (or just implement these changes into meval-rs, idk)
    // assert_eq!(&add_asterisks("emax(x)".to_string()), "e*max(x)");
    // assert_eq!(&add_asterisks("pisin(x)".to_string()), "pi*sin(x)");
}

// Tests cache when initialized with value
#[test]
fn cache_test_full() {
    let mut cache = Cache::new("data");
    assert!(cache.is_valid());
    cache.invalidate();
    assert_eq!(cache.is_valid(), false);
    cache.set("data2");
    assert!(cache.is_valid());
}

// Tests cache when initialized without value
#[test]
fn cache_test_empty() {
    let mut cache: Cache<&str> = Cache::new_empty();
    assert_eq!(cache.is_valid(), false);
    cache.invalidate();
    assert_eq!(cache.is_valid(), false);
    cache.set("data");
    assert!(cache.is_valid());
}
