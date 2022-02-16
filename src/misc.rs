use wasm_bindgen::prelude::*;

pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

// EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed. This is so cursed
pub fn add_asterisks(function: String) -> String {
    let letters: Vec<char>= "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let numbers: Vec<char> = "0123456789".chars().collect();
    let function_chars: Vec<char> = function.chars().collect();
    let func_chars_len = function_chars.len();
    let mut output_string: String = String::new();
    let mut prev_chars: Vec<char> = Vec::new();
    for c in function_chars.clone() {
        let prev_chars_len = prev_chars.len();
        let curr_i: usize = func_chars_len-prev_chars_len;

        let prev_prev_char = if prev_chars_len >= 2 {
            match prev_chars.get(prev_chars_len-2) {
                Some(x) => *x,
                None => panic!()
            }
        } else {
            ' '
        };

        let prev_char = if prev_chars_len >= 1 {
            match prev_chars.get(prev_chars_len-1) {
                Some(x) => *x,
                None => panic!()
            }
        } else {
            ' '
        };

        let for_char =  match function_chars.get(curr_i) {
            Some(x) => *x,
            None => ' ',
        };

        let prev_pi = (prev_prev_char == 'p') && (prev_char == 'i');
        let for_pi = (for_char == 'i') && (c == 'p');

        if prev_char == ')' {
            if c == '(' {
                output_string += "*";
            } else if (c == 'x') | (c == 'e') | numbers.contains(&c) | for_pi {
                output_string += "*";
            }
        } else if c == '(' {
            if ((prev_char == 'x') | (prev_char == 'e') | (prev_char == ')') | numbers.contains(&prev_char)) && !letters.contains(&prev_prev_char) {
                output_string += "*";
            } else if prev_pi {
                output_string += "*";
            }
        } else if numbers.contains(&prev_char) {
            if (c == '(') | letters.contains(&c) | for_pi {
                output_string += "*";
            }
        } else if letters.contains(&c) {
            if numbers.contains(&prev_char) {
                output_string += "*";
            } else if ((c == 'x') | (c == 'e')) && ((prev_char == 'x') | (prev_char == 'e')) | prev_pi {
                output_string += "*";
            }
        } else if numbers.contains(&c) {
            if letters.contains(&prev_char) | prev_pi {
                output_string += "*";
            }
        }

        prev_chars.push(c);
        output_string += &c.to_string();
    }

    return output_string;
}

/// Result of screen to chart coordinates conversion.
#[wasm_bindgen]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[wasm_bindgen]
impl Point {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

#[wasm_bindgen]
pub struct ChartOutput {
    pub(crate) convert: Box<dyn Fn((i32, i32)) -> Option<(f32, f32)>>,
    pub(crate) area: f32,
}

#[wasm_bindgen]
impl ChartOutput {
    pub fn get_area(&self) -> f32 { self.area }

    pub fn coord(&self, x: i32, y: i32) -> Option<Point> {
        (self.convert)((x, y)).map(|(x, y)| Point::new(x, y))
    }
}

pub struct Cache<T> {
    backing_data: Option<T>,
    valid: bool,
}

impl<T> Cache<T> {
    #[allow(dead_code)]
    #[inline]
    pub fn new(backing_data: T) -> Self {
        Self {
            backing_data: Some(backing_data),
            valid: true,
        }
    }

    #[inline]
    pub fn new_empty() -> Self {
        Self {
            backing_data: None,
            valid: false,
        }
    }

    #[inline]
    pub fn get(&self) -> &T {
        if !self.valid {
            panic!("self.valid is false, but get() method was called!")
        }

        match &self.backing_data {
            Some(x) => x,
            None => panic!("self.backing_data is None"),
        }
    }

    #[inline]
    pub fn set(&mut self, data: T) {
        self.valid = true;
        self.backing_data = Some(data);
    }

    #[inline]
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.backing_data = None;
    }

    #[inline]
    pub fn is_valid(&self) -> bool { self.valid }
}

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
    assert_eq!(&add_asterisks("xxx".to_string()), "x*x*x");
    assert_eq!(&add_asterisks("eee".to_string()), "e*e*e");
    assert_eq!(&add_asterisks("pi(x+2)".to_string()), "pi*(x+2)");
    assert_eq!(&add_asterisks("(x)pi".to_string()), "(x))*pi");
    assert_eq!(&add_asterisks("2e".to_string()), "2*e");
}