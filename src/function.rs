#![allow(clippy::too_many_arguments)] // Clippy, shut

#[allow(unused_imports)]
use crate::misc::{debug_log, SteppedVector};

use eframe::egui::{
    plot::{BarChart, Line, Value, Values},
    widgets::plot::Bar,
};
use meval::Expr;
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

pub struct Function {
    function: Box<dyn Fn(f64) -> f64>,
    func_str: String,
    min_x: f64,
    max_x: f64,
    pixel_width: usize,

    back_cache: Option<Vec<Value>>,
    front_cache: Option<(Vec<Bar>, Vec<Value>, f64)>,
    derivative_cache: Option<Vec<Value>>,

    pub(crate) integral: bool,
    pub(crate) derivative: bool,
    integral_min_x: f64,
    integral_max_x: f64,
    integral_num: usize,
    sum: RiemannSum,
}

// x^2 function, set here so we don't have to regenerate it every time a new function is made
fn default_function(x: f64) -> f64 { x.powi(2) }

const EPSILON: f64 = 5.0e-7;

impl Function {
    // Creates Empty Function instance
    pub fn empty() -> Self {
        Self {
            function: Box::new(default_function),
            func_str: String::new(),
            min_x: -1.0,
            max_x: 1.0,
            pixel_width: 100,
            back_cache: None,
            front_cache: None,
            derivative_cache: None,
            integral: false,
            derivative: false,
            integral_min_x: f64::NAN,
            integral_max_x: f64::NAN,
            integral_num: 0,
            sum: crate::egui_app::DEFAULT_RIEMANN,
        }
    }

    // Runs the internal function to get values
    fn run_func(&self, x: f64) -> f64 { (self.function)(x) }

    pub fn update(
        &mut self, func_str: String, integral: bool, derivative: bool, integral_min_x: Option<f64>,
        integral_max_x: Option<f64>, integral_num: Option<usize>, sum: Option<RiemannSum>,
    ) {
        // If the function string changes, just wipe and restart from scratch
        if func_str != self.func_str {
            self.func_str = func_str.clone();
            self.function = Box::new({
                let expr: Expr = func_str.parse().unwrap();
                expr.bind("x").unwrap()
            });
            self.back_cache = None;
            self.front_cache = None;
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
            self.front_cache = None;
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
                            Value::new(x, self.run_func(x))
                        }
                    })
                    .collect(),
            );

            if self.derivative_cache.is_some() {
                let derivative_cache = self.derivative_cache.as_ref().unwrap();

                self.derivative_cache = Some(
                    (0..=self.pixel_width)
                        .map(|x| (x as f64 / resolution as f64) + min_x)
                        .map(|x| {
                            if let Some(i) = x_data.get_index(x) {
                                derivative_cache[i]
                            } else {
                                let (x1, x2) = (x - EPSILON, x + EPSILON);
                                let (y1, y2) = (self.run_func(x1), self.run_func(x2));
                                let slope = (y2 - y1) / (EPSILON * 2.0);
                                Value::new(x, slope)
                            }
                        })
                        .collect(),
                );
            }
        } else {
            self.back_cache = None;
            self.derivative_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        }
    }

    pub fn run_back(
        &mut self,
    ) -> (
        Vec<Value>,
        Option<(Vec<Bar>, Vec<Value>, f64)>,
        Option<Vec<Value>>,
    ) {
        let back_values: Vec<Value> = {
            if self.back_cache.is_none() {
                let resolution: f64 =
                    (self.pixel_width as f64 / (self.max_x - self.min_x).abs()) as f64;
                self.back_cache = Some(
                    (0..self.pixel_width)
                        .map(|x| (x as f64 / resolution as f64) + self.min_x)
                        .map(|x| Value::new(x, self.run_func(x)))
                        .collect(),
                );
            }

            self.back_cache.as_ref().unwrap().clone()
        };

        let derivative_values: Option<Vec<Value>> = match self.derivative {
            true => {
                if self.derivative_cache.is_none() {
                    let back_cache = self.back_cache.as_ref().unwrap().clone();
                    self.derivative_cache = Some(
                        back_cache
                            .iter()
                            .map(|ele| {
                                let x = ele.x;
                                let (x1, x2) = (x - EPSILON, x + EPSILON);
                                let (y1, y2) = (self.run_func(x1), self.run_func(x2));
                                let slope = (y2 - y1) / (EPSILON * 2.0);
                                Value::new(x, slope)
                            })
                            .collect(),
                    );
                }
                Some(self.derivative_cache.as_ref().unwrap().clone())
            }
            false => None,
        };

        let front_bars = match self.integral {
            true => {
                if self.front_cache.is_none() {
                    let (data, area) = self.integral_rectangles();
                    self.front_cache = Some((
                        data.iter().map(|(x, y, _)| Bar::new(*x, *y)).collect(),
                        data.iter().map(|(x, _, y)| Value::new(*x, *y)).collect(),
                        area,
                    ));
                }
                let cache = self.front_cache.as_ref().unwrap();
                Some((cache.0.clone(), cache.1.clone(), cache.2))
            }
            false => None,
        };

        (back_values, front_bars, derivative_values)
    }

    pub fn run(&mut self) -> (Line, Option<(BarChart, Line, f64)>, Option<Line>) {
        let (back_values, front_data_option, derivative_option) = self.run_back();

        (
            Line::new(Values::from_values(back_values)),
            if let Some(front_data1) = front_data_option {
                Some((
                    BarChart::new(front_data1.0),
                    Line::new(Values::from_values(front_data1.1)),
                    front_data1.2,
                ))
            } else {
                None
            },
            derivative_option
                .map(|derivative_data| Line::new(Values::from_values(derivative_data))),
        )
    }

    // Creates and does the math for creating all the rectangles under the graph
    fn integral_rectangles(&self) -> (Vec<(f64, f64, f64)>, f64) {
        if self.integral_min_x.is_nan() {
            panic!("integral_min_x is NaN")
        } else if self.integral_max_x.is_nan() {
            panic!("integral_max_x is NaN")
        }

        let step = (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64);

        let mut area: f64 = 0.0;
        let data2: Vec<(f64, f64, f64)> = (1..=self.integral_num)
            .map(|e| {
                let x: f64 = ((e as f64) * step) + self.integral_min_x;
                let step_offset = step * x.signum(); // store the offset here so it doesn't have to be calculated multiple times
                let x2: f64 = x + step_offset;

                let (left_x, right_x) = match x.is_sign_positive() {
                    true => (x, x2),
                    false => (x2, x),
                };

                let y = match self.sum {
                    RiemannSum::Left => self.run_func(left_x),
                    RiemannSum::Right => self.run_func(right_x),
                    RiemannSum::Middle => (self.run_func(left_x) + self.run_func(right_x)) / 2.0,
                };

                if !y.is_nan() {
                    area += y * step;
                }

                (x + (step_offset / 2.0), y, area)
            })
            .filter(|(_, y, _)| !y.is_nan())
            .collect();
        (data2, area)
    }

    // Set func_str to an empty string
    pub fn empty_func_str(&mut self) { self.func_str = String::new(); }

    // Updates riemann value and invalidates front_cache if needed
    pub fn update_riemann(mut self, riemann: RiemannSum) -> Self {
        if self.sum != riemann {
            self.sum = riemann;
            self.front_cache = None;
        }
        self
    }

    // Toggles integral
    pub fn integral(mut self, integral: bool) -> Self {
        self.integral = integral;
        self
    }
}

#[test]
fn left_function_test() {
    let pixel_width = 10;
    let integral_num = 10;

    let mut function = Function {
        function: Box::new(default_function),
        func_str: String::from("x^2"),
        min_x: -1.0,
        max_x: 1.0,
        pixel_width,
        back_cache: None,
        front_cache: None,
        derivative_cache: None,
        integral: false,
        derivative: false,
        integral_min_x: -1.0,
        integral_max_x: 1.0,
        integral_num,
        sum: RiemannSum::Left,
    };

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

    let derivative_target = vec![
        (-1.0, -2.0000000000575113),
        (-0.8, -1.5999999999349868),
        (-0.6, -1.1999999998679733),
        (-0.4, -0.8000000000230045),
        (-0.19999999999999996, -0.3999999999906856),
        (0.0, 0.0),
        (0.19999999999999996, 0.3999999999906856),
        (0.3999999999999999, 0.8000000000230045),
        (0.6000000000000001, 1.1999999999234845),
        (0.8, 1.5999999999349868),
    ];

    let area_target = 0.8720000000000001;

    let vec_bars_target = vec![
        1.0,
        0.6400000000000001,
        0.3599999999999998,
        0.15999999999999998,
        0.0,
        0.04000000000000007,
        0.16000000000000011,
        0.3600000000000001,
        0.6400000000000001,
        1.0,
    ];

    let vec_integral_target = vec![
        (-0.9, 0.2),
        (-0.7, 0.32800000000000007),
        (-0.4999999999999999, 0.4),
        (-0.29999999999999993, 0.432),
        (0.1, 0.432),
        (0.30000000000000016, 0.44),
        (0.5000000000000001, 0.47200000000000003),
        (0.7000000000000001, 0.544),
        (0.9, 0.672),
        (1.1, 0.8720000000000001),
    ];

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

        assert_eq!(bars.clone().unwrap().2, area_target);

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
        let derivative_vec: Vec<(f64, f64)> = derivative
            .unwrap()
            .iter()
            .map(|ele| (ele.x, ele.y))
            .collect();
        assert_eq!(derivative_vec, derivative_target);

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().2, area_target);
        let bars_unwrapped = bars.unwrap();

        let vec_bars: Vec<f64> = bars_unwrapped.0.iter().map(|bar| bar.value).collect();

        assert_eq!(vec_bars.len(), integral_num);
        assert_eq!(vec_bars, vec_bars_target);

        let integral_line = bars_unwrapped.1;
        let vec_integral: Vec<(f64, f64)> =
            integral_line.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(vec_integral.len(), integral_num);

        assert_eq!(vec_integral, vec_integral_target);
        assert_eq!(vec_integral[vec_integral.len()-1].1, area_target);
    }
}

#[test]
fn middle_function_test() {
    let pixel_width = 10;
    let integral_num = 10;

    let mut function = Function {
        function: Box::new(default_function),
        func_str: String::from("x^2"),
        min_x: -1.0,
        max_x: 1.0,
        pixel_width,
        back_cache: None,
        front_cache: None,
        derivative_cache: None,
        integral: false,
        derivative: false,
        integral_min_x: -1.0,
        integral_max_x: 1.0,
        integral_num,
        sum: RiemannSum::Middle,
    };

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

    let derivative_target = vec![
        (-1.0, -2.0000000000575113),
        (-0.8, -1.5999999999349868),
        (-0.6, -1.1999999998679733),
        (-0.4, -0.8000000000230045),
        (-0.19999999999999996, -0.3999999999906856),
        (0.0, 0.0),
        (0.19999999999999996, 0.3999999999906856),
        (0.3999999999999999, 0.8000000000230045),
        (0.6000000000000001, 1.1999999999234845),
        (0.8, 1.5999999999349868),
    ];

    let area_target = 0.9200000000000002;

    let vec_bars_target = vec![
        0.8200000000000001,
        0.5,
        0.2599999999999999,
        0.09999999999999998,
        0.020000000000000004,
        0.1000000000000001,
        0.2600000000000001,
        0.5000000000000001,
        0.8200000000000001,
        1.22,
    ];

    let vec_integral_target = vec![
        (-0.9, 0.16400000000000003),
        (-0.7, 0.264),
        (-0.4999999999999999, 0.316),
        (-0.29999999999999993, 0.336),
        (0.1, 0.34),
        (0.30000000000000016, 0.36000000000000004),
        (0.5000000000000001, 0.4120000000000001),
        (0.7000000000000001, 0.5120000000000001),
        (0.9, 0.6760000000000002),
        (1.1, 0.9200000000000002),
    ];

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

        assert_eq!(bars.clone().unwrap().2, area_target);

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
        let derivative_vec: Vec<(f64, f64)> = derivative
            .unwrap()
            .iter()
            .map(|ele| (ele.x, ele.y))
            .collect();
        assert_eq!(derivative_vec, derivative_target);

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().2, area_target);
        let bars_unwrapped = bars.unwrap();

        let vec_bars: Vec<f64> = bars_unwrapped.0.iter().map(|bar| bar.value).collect();

        assert_eq!(vec_bars.len(), integral_num);
        assert_eq!(vec_bars, vec_bars_target);

        let integral_line = bars_unwrapped.1;
        let vec_integral: Vec<(f64, f64)> =
            integral_line.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(vec_integral.len(), integral_num);

        assert_eq!(vec_integral, vec_integral_target);
        assert_eq!(vec_integral[vec_integral.len()-1].1, area_target);
    }
}

#[test]
fn right_function_test() {
    let pixel_width = 10;
    let integral_num = 10;

    let mut function = Function {
        function: Box::new(default_function),
        func_str: String::from("x^2"),
        min_x: -1.0,
        max_x: 1.0,
        pixel_width,
        back_cache: None,
        front_cache: None,
        derivative_cache: None,
        integral: false,
        derivative: false,
        integral_min_x: -1.0,
        integral_max_x: 1.0,
        integral_num,
        sum: RiemannSum::Right,
    };

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

    let derivative_target = vec![
        (-1.0, -2.0000000000575113),
        (-0.8, -1.5999999999349868),
        (-0.6, -1.1999999998679733),
        (-0.4, -0.8000000000230045),
        (-0.19999999999999996, -0.3999999999906856),
        (0.0, 0.0),
        (0.19999999999999996, 0.3999999999906856),
        (0.3999999999999999, 0.8000000000230045),
        (0.6000000000000001, 1.1999999999234845),
        (0.8, 1.5999999999349868),
    ];

    let area_target = 0.9680000000000002;

    let vec_bars_target = vec![
        0.6400000000000001,
        0.36,
        0.15999999999999992,
        0.03999999999999998,
        0.04000000000000001,
        0.16000000000000014,
        0.3600000000000001,
        0.6400000000000001,
        1.0,
        1.44,
    ];

    let vec_integral_target = vec![
        (-0.9, 0.12800000000000003),
        (-0.7, 0.2),
        (-0.4999999999999999, 0.23199999999999998),
        (-0.29999999999999993, 0.24),
        (0.1, 0.248),
        (0.30000000000000016, 0.28),
        (0.5000000000000001, 0.35200000000000004),
        (0.7000000000000001, 0.4800000000000001),
        (0.9, 0.6800000000000002),
        (1.1, 0.9680000000000002),
    ];

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

        assert_eq!(bars.clone().unwrap().2, area_target);

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
        let derivative_vec: Vec<(f64, f64)> = derivative
            .unwrap()
            .iter()
            .map(|ele| (ele.x, ele.y))
            .collect();
        assert_eq!(derivative_vec, derivative_target);

        assert!(bars.is_some());
        assert_eq!(back_values.len(), pixel_width);
        assert_eq!(bars.clone().unwrap().2, area_target);
        let bars_unwrapped = bars.unwrap();

        let vec_bars: Vec<f64> = bars_unwrapped.0.iter().map(|bar| bar.value).collect();

        assert_eq!(vec_bars.len(), integral_num);
        assert_eq!(vec_bars, vec_bars_target);

        let integral_line = bars_unwrapped.1;
        let vec_integral: Vec<(f64, f64)> =
            integral_line.iter().map(|ele| (ele.x, ele.y)).collect();
        assert_eq!(vec_integral.len(), integral_num);

        assert_eq!(vec_integral, vec_integral_target);
        assert_eq!(vec_integral[vec_integral.len()-1].1, area_target);
    }
}
