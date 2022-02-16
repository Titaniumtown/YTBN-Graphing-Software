use wasm_bindgen::prelude::*;

pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

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
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}
