#![allow(clippy::too_many_arguments)] // Clippy, shut

#[allow(unused_imports)]
use crate::misc::{debug_log, SteppedVector};

use crate::egui_app::{DEFAULT_FUNCION, DEFAULT_RIEMANN};
use crate::parsing::BackingFunction;

use eframe::egui::{
    plot::{BarChart, Line, Value, Values},
    widgets::plot::Bar,
};
use std::fmt::{self, Debug};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RiemannSum {
    Left,
    Middle,
    Right,
}

impl fmt::Display for RiemannSum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

lazy_static::lazy_static! {
    pub static ref EMPTY_FUNCTIONENTRY: FunctionEntry = FunctionEntry::empty();
}

#[derive(Clone)]
pub struct FunctionEntry {
    function: BackingFunction,
    func_str: String,
    min_x: f64,
    max_x: f64,
    pixel_width: usize,

    back_cache: Option<Vec<Value>>,
    integral_cache: Option<(Vec<Bar>, f64)>,
    derivative_cache: Option<Vec<Value>>,

    pub(crate) integral: bool,
    pub(crate) derivative: bool,
    integral_min_x: f64,
    integral_max_x: f64,
    integral_num: usize,
    sum: RiemannSum,
}

impl FunctionEntry {
    // Creates Empty Function instance
    pub fn empty() -> Self {
        Self {
            function: BackingFunction::new(DEFAULT_FUNCION),
            func_str: String::new(),
            min_x: -1.0,
            max_x: 1.0,
            pixel_width: 100,
            back_cache: None,
            integral_cache: None,
            derivative_cache: None,
            integral: false,
            derivative: false,
            integral_min_x: f64::NAN,
            integral_max_x: f64::NAN,
            integral_num: 0,
            sum: DEFAULT_RIEMANN,
        }
    }

    pub fn update(
        &mut self, func_str: String, integral: bool, derivative: bool, integral_min_x: Option<f64>,
        integral_max_x: Option<f64>, integral_num: Option<usize>, sum: Option<RiemannSum>,
    ) {
        // If the function string changes, just wipe and restart from scratch
        if func_str != self.func_str {
            self.func_str = func_str.clone();
            self.function = BackingFunction::new(&func_str);
            self.back_cache = None;
            self.integral_cache = None;
            self.derivative_cache = None;
        }

        self.derivative = derivative;
        self.integral = integral;

        // Makes sure proper arguments are passed when integral is enabled
        if integral
            && (integral_min_x != Some(self.integral_min_x))
                | (integral_max_x != Some(self.integral_max_x))
                | (integral_num != Some(self.integral_num))
                | (sum != Some(self.sum))
        {
            self.integral_cache = None;
            self.integral_min_x = integral_min_x.expect("integral_min_x is None");
            self.integral_max_x = integral_max_x.expect("integral_max_x is None");
            self.integral_num = integral_num.expect("integral_num is None");
            self.sum = sum.expect("sum is None");
        }
    }

    pub fn update_bounds(&mut self, min_x: f64, max_x: f64, pixel_width: usize) {
        if pixel_width != self.pixel_width {
            self.back_cache = None;
            self.derivative_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        } else if ((min_x != self.min_x) | (max_x != self.max_x)) && self.back_cache.is_some() {
            let resolution: f64 = self.pixel_width as f64 / (max_x.abs() + min_x.abs());
            let back_cache = self.back_cache.as_ref().unwrap();

            let x_data: SteppedVector = back_cache
                .iter()
                .map(|ele| ele.x)
                .collect::<Vec<f64>>()
                .into();

            self.back_cache = Some(
                (0..=self.pixel_width)
                    .map(|x| (x as f64 / resolution as f64) + min_x)
                    .map(|x| {
                        if let Some(i) = x_data.get_index(x) {
                            back_cache[i]
                        } else {
                            Value::new(x, self.function.get(x))
                        }
                    })
                    .collect(),
            );

            if self.derivative_cache.is_some() {
                if self.derivative {
                    let derivative_cache = self.derivative_cache.as_ref().unwrap();

                    self.derivative_cache = Some(
                        (0..=self.pixel_width)
                            .map(|x| (x as f64 / resolution as f64) + min_x)
                            .map(|x| {
                                if let Some(i) = x_data.get_index(x) {
                                    derivative_cache[i]
                                } else {
                                    Value::new(x, self.function.derivative(x))
                                }
                            })
                            .collect(),
                    );
                } else {
                    self.derivative_cache = None;
                }
            }
        } else {
            self.back_cache = None;
            self.derivative_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        }
    }

    pub fn run_back(&mut self) -> (Vec<Value>, Option<(Vec<Bar>, f64)>, Option<Vec<Value>>) {
        let resolution: f64 = (self.pixel_width as f64 / (self.max_x - self.min_x).abs()) as f64;
        let back_values: Vec<Value> = {
            if self.back_cache.is_none() {
                self.back_cache = Some(
                    (0..self.pixel_width)
                        .map(|x| (x as f64 / resolution as f64) + self.min_x)
                        .map(|x| Value::new(x, self.function.get(x)))
                        .collect(),
                );
            }

            self.back_cache.as_ref().unwrap().clone()
        };

        let derivative_values: Option<Vec<Value>> = match self.derivative {
            true => {
                if self.derivative_cache.is_none() {
                    self.derivative_cache = Some(
                        (0..self.pixel_width)
                            .map(|x| (x as f64 / resolution as f64) + self.min_x)
                            .map(|x| Value::new(x, self.function.derivative(x)))
                            .collect(),
                    );
                }
                Some(self.derivative_cache.as_ref().unwrap().clone())
            }
            false => None,
        };

        let integral_data = match self.integral {
            true => {
                if self.integral_cache.is_none() {
                    let (data, area) = self.integral_rectangles();
                    self.integral_cache =
                        Some((data.iter().map(|(x, y)| Bar::new(*x, *y)).collect(), area));
                }
                let cache = self.integral_cache.as_ref().unwrap();
                Some((cache.0.clone(), cache.1))
            }
            false => None,
        };

        (back_values, integral_data, derivative_values)
    }

    pub fn run(&mut self) -> (Line, Option<(BarChart, f64)>, Option<Line>) {
        let (back_values, integral_data_option, derivative_option) = self.run_back();

        (
            Line::new(Values::from_values(back_values)),
            if let Some(integral_data) = integral_data_option {
                Some((BarChart::new(integral_data.0), integral_data.1))
            } else {
                None
            },
            derivative_option
                .map(|derivative_data| Line::new(Values::from_values(derivative_data))),
        )
    }

    // Creates and does the math for creating all the rectangles under the graph
    fn integral_rectangles(&self) -> (Vec<(f64, f64)>, f64) {
        if self.integral_min_x.is_nan() {
            panic!("integral_min_x is NaN")
        } else if self.integral_max_x.is_nan() {
            panic!("integral_max_x is NaN")
        }

        let step = (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64);

        let mut last_positive: Option<bool> = None;
        let mut area: f64 = 0.0;
        let data2: Vec<(f64, f64)> = (0..self.integral_num)
            .map(|e| {
                let x: f64 = ((e as f64) * step) + self.integral_min_x;
                let step_offset = step * x.signum(); // store the offset here so it doesn't have to be calculated multiple times
                let x2: f64 = x + step_offset;

                let (left_x, right_x) = match x.is_sign_positive() {
                    true => (x, x2),
                    false => (x2, x),
                };

                let y = match self.sum {
                    RiemannSum::Left => self.function.get(left_x),
                    RiemannSum::Right => self.function.get(right_x),
                    RiemannSum::Middle => {
                        (self.function.get(left_x) + self.function.get(right_x)) / 2.0
                    }
                };

                if last_positive.is_none() {
                    last_positive = Some(x.is_sign_positive());
                }

                if !y.is_nan() {
                    area += y * step;
                }

                (x + (step_offset / 2.0), y)
            })
            .filter(|(_, y)| !y.is_nan())
            .collect();
        (data2, area)
    }

    pub fn get_func_str(&self) -> &str { &self.func_str }

    // Updates riemann value and invalidates integral_cache if needed
    pub fn update_riemann(mut self, riemann: RiemannSum) -> Self {
        if self.sum != riemann {
            self.sum = riemann;
            self.integral_cache = None;
        }
        self
    }

    // Toggles integral
    pub fn integral(mut self, integral: bool) -> Self {
        self.integral = integral;
        self
    }

    #[allow(dead_code)]
    pub fn integral_num(mut self, integral_num: usize) -> Self {
        self.integral_num = integral_num;
        self
    }

    #[allow(dead_code)]
    pub fn pixel_width(mut self, pixel_width: usize) -> Self {
        self.pixel_width = pixel_width;
        self
    }

    #[allow(dead_code)]
    pub fn integral_bounds(mut self, min_x: f64, max_x: f64) -> Self {
        if min_x >= max_x {
            panic!("integral_bounds: min_x is larger than max_x");
        }

        self.integral_min_x = min_x;
        self.integral_max_x = max_x;
        self
    }

    pub fn get_derivative_str(&self) -> String { self.function.get_derivative_str() }
}

#[test]
fn left_function_test() {
    let integral_num = 10;
    let pixel_width = 10;

    let mut function = FunctionEntry::empty()
        .update_riemann(RiemannSum::Left)
        .pixel_width(pixel_width)
        .integral_num(integral_num)
        .integral_bounds(-1.0, 1.0);

    let back_values_target = vec![
        (-1.0, 1.0),
        (-0.8, 0.6400000000000001),
        (-0.6, 0.36),
        (-0.4, 0.16000000000000003),
        (-0.19999999999999996, 0.03999999999999998),
        (0.0, 0.0),
        (0.19999999999999996, 0.03999999999999998),
        (0.3999999999999999, 0.15999999999999992),
        (0.6000000000000001, 0.3600000000000001),
        (0.8, 0.6400000000000001),
    ];

    let area_target = 0.9600000000000001;

    {
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_none());
        assert_eq!(back_values.len(), pixel_width);
        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function = function.integral(true);
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);

        assert_eq!(bars.clone().unwrap().1, area_target);

        let vec_bars = bars.unwrap().0;
        assert_eq!(vec_bars.len(), integral_num);

        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function.derivative = true;
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_some());

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().1, area_target);
        let bars_unwrapped = bars.unwrap();

        assert_eq!(bars_unwrapped.0.iter().len(), integral_num);
    }
}

#[test]
fn middle_function_test() {
    let integral_num = 10;
    let pixel_width = 10;

    let mut function = FunctionEntry::empty()
        .update_riemann(RiemannSum::Middle)
        .pixel_width(pixel_width)
        .integral_num(integral_num)
        .integral_bounds(-1.0, 1.0);

    let back_values_target = vec![
        (-1.0, 1.0),
        (-0.8, 0.6400000000000001),
        (-0.6, 0.36),
        (-0.4, 0.16000000000000003),
        (-0.19999999999999996, 0.03999999999999998),
        (0.0, 0.0),
        (0.19999999999999996, 0.03999999999999998),
        (0.3999999999999999, 0.15999999999999992),
        (0.6000000000000001, 0.3600000000000001),
        (0.8, 0.6400000000000001),
    ];

    let area_target = 0.92;

    {
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_none());
        assert_eq!(back_values.len(), pixel_width);
        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function = function.integral(true);
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);

        assert_eq!(bars.clone().unwrap().1, area_target);

        let vec_bars = bars.unwrap().0;
        assert_eq!(vec_bars.len(), integral_num);

        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function.derivative = true;
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_some());

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().1, area_target);
        let bars_unwrapped = bars.unwrap();

        assert_eq!(bars_unwrapped.0.iter().len(), integral_num);
    }
}

#[test]
fn right_function_test() {
    let integral_num = 10;
    let pixel_width = 10;

    let mut function = FunctionEntry::empty()
        .update_riemann(RiemannSum::Right)
        .pixel_width(pixel_width)
        .integral_num(integral_num)
        .integral_bounds(-1.0, 1.0);

    let back_values_target = vec![
        (-1.0, 1.0),
        (-0.8, 0.6400000000000001),
        (-0.6, 0.36),
        (-0.4, 0.16000000000000003),
        (-0.19999999999999996, 0.03999999999999998),
        (0.0, 0.0),
        (0.19999999999999996, 0.03999999999999998),
        (0.3999999999999999, 0.15999999999999992),
        (0.6000000000000001, 0.3600000000000001),
        (0.8, 0.6400000000000001),
    ];

    let area_target = 0.8800000000000001;

    {
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_none());
        assert_eq!(back_values.len(), pixel_width);
        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function = function.integral(true);
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_none());
        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);

        assert_eq!(bars.clone().unwrap().1, area_target);

        let vec_bars = bars.unwrap().0;
        assert_eq!(vec_bars.len(), integral_num);

        let back_values_tuple: Vec<(f64, f64)> =
            back_values.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(back_values_tuple, back_values_target);
    }

    {
        function.derivative = true;
        let (back_values, bars, derivative) = function.run_back();
        assert!(derivative.is_some());

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().1, area_target);
        let bars_unwrapped = bars.unwrap();

        assert_eq!(bars_unwrapped.0.iter().len(), integral_num);
    }
}
