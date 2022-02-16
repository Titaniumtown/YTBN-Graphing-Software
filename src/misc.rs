use wasm_bindgen::prelude::*;
use meval::Expr;

pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn test_func_basic(function_string: String) -> String {
    let expr_result = function_string.parse();
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


// EXTREMELY Janky function that tries to put asterisks in the proper places to be parsed
pub fn add_asterisks(function: String) -> String {
    let letters: Vec<char>= "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let numbers: Vec<char> = "0123456789".chars().collect();
    let function_chars = function.chars();
    let mut output_string: String = String::new();
    let mut prev_char: char = ' ';
    for c in function_chars {
        if prev_char == ')' {
            if c == '(' {
                output_string += "*";
            } else if (c == 'x') | numbers.contains(&c) {
                output_string += "*"
            }
        } else if letters.contains(&c) {
            if numbers.contains(&prev_char) {
                output_string += "*";
            }
        } else if numbers.contains(&c) {
            if letters.contains(&prev_char) {
                output_string += "*";
            }
        }
        prev_char = c;
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
