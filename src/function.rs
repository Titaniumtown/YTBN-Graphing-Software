use crate::misc::debug_log;
use eframe::egui::{plot::Value, widgets::plot::Bar};
use meval::Expr;

// Struct that stores and manages the output of a function
pub struct FunctionOutput {
    // The actual line graph
    back: Vec<Value>,

    // Integral information
    front: Option<(Vec<Bar>, f64)>,
}

impl FunctionOutput {
    pub fn new(back: Vec<Value>, front: Option<(Vec<Bar>, f64)>) -> Self { Self { back, front } }

    pub fn get_back(&self) -> Vec<Value> { self.back.clone() }

    pub fn get_front(&self) -> (Vec<Bar>, f64) {
        match &self.front {
            Some(x) => (x.0.clone(), x.1),
            None => panic!(""),
        }
    }

    pub fn has_integral(&self) -> bool { self.front.is_some() }
}

pub struct Function {
    function: Box<dyn Fn(f64) -> f64>,
    pub(crate) func_str: String,
    min_x: f64,
    max_x: f64,
    pixel_width: usize,

    back_cache: Option<Vec<Value>>,
    front_cache: Option<(Vec<Bar>, f64)>,

    integral: bool,
    integral_min_x: f64,
    integral_max_x: f64,
    integral_num: usize,
}

impl Clone for Function {
    fn clone(&self) -> Self {
        let expr: Expr = self.func_str.parse().unwrap();
        let func = expr.bind("x").unwrap();
        Self {
            function: Box::new(func),
            func_str: self.func_str.clone(),
            min_x: self.min_x,
            max_x: self.max_x,
            pixel_width: self.pixel_width,
            back_cache: self.back_cache.clone(),
            front_cache: self.front_cache.clone(),
            integral: self.integral,
            integral_min_x: self.integral_min_x,
            integral_max_x: self.integral_max_x,
            integral_num: self.integral_num,
        }
    }
}

impl Function {
    pub fn new(
        func_str: String, min_x: f64, max_x: f64, pixel_width: usize, integral: bool,
        integral_min_x: Option<f64>, integral_max_x: Option<f64>, integral_num: Option<usize>,
    ) -> Self {
        // Makes sure proper arguments are passed when integral is enabled
        if integral {
            if integral_min_x.is_none() {
                panic!("Invalid arguments: integral_min_x is None, but integral is enabled.")
            } else if integral_max_x.is_none() {
                panic!("Invalid arguments: integral_max_x is None, but integral is enabled.")
            } else if integral_num.is_none() {
                panic!("Invalid arguments: integral_num is None, but integral is enabled.")
            }
        }

        let expr: Expr = func_str.parse().unwrap();
        let func = expr.bind("x").unwrap();
        Self {
            function: Box::new(func),
            func_str,
            min_x,
            max_x,
            pixel_width,
            back_cache: None,
            front_cache: None,
            integral,
            integral_min_x: match integral_min_x {
                Some(x) => x,
                None => f64::NAN,
            },
            integral_max_x: match integral_max_x {
                Some(x) => x,
                None => f64::NAN,
            },
            integral_num: integral_num.unwrap_or(0),
        }
    }

    // Runs the internal function to get values
    fn run_func(&self, x: f64) -> f64 { (self.function)(x) }

    pub fn update(
        &mut self, func_str: String, integral: bool, integral_min_x: Option<f64>,
        integral_max_x: Option<f64>, integral_num: Option<usize>,
    ) {
        if func_str.is_empty() {
            self.func_str = func_str;
            return;
        }

        // If the function string changes, just wipe and restart from scratch
        if func_str != self.func_str {
            *self = Self::new(
                func_str,
                self.min_x,
                self.max_x,
                self.pixel_width,
                integral,
                integral_min_x,
                integral_max_x,
                integral_num,
            );
            return;
        }

        if integral != self.integral {
            self.integral = integral;
        }

        // Makes sure proper arguments are passed when integral is enabled
        if integral {
            if integral_min_x.is_none() {
                panic!("Invalid arguments: integral_min_x is None, but integral is enabled.")
            } else if integral_max_x.is_none() {
                panic!("Invalid arguments: integral_max_x is None, but integral is enabled.")
            } else if integral_num.is_none() {
                panic!("Invalid arguments: integral_num is None, but integral is enabled.")
            }

            if (integral_min_x != Some(self.integral_min_x))
                | (integral_max_x != Some(self.integral_max_x))
                | (integral_num != Some(self.integral_num))
            {
                self.front_cache = None;
                self.integral_min_x = integral_min_x.expect("");
                self.integral_max_x = integral_max_x.expect("");
                self.integral_num = integral_num.expect("");
            }
        }
    }

    pub fn update_bounds(&mut self, min_x: f64, max_x: f64, pixel_width: usize) {
        if pixel_width != self.pixel_width {
            self.back_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        } else if ((min_x != self.min_x) | (max_x != self.max_x)) && self.back_cache.is_some() {
            // debug_log("back_cache: partial regen");
            let range_new: f64 = max_x.abs() + min_x.abs();

            let resolution: f64 = (self.pixel_width as f64 / range_new) as f64;
            let movement_right = min_x > self.min_x;
            let mut new_back: Vec<Value> = self
                .back_cache
                .as_ref()
                .expect("")
                .clone()
                .iter()
                .filter(|ele| (ele.x >= min_x) && (min_x >= ele.x))
                .copied()
                .collect();

            let x_to_go = match movement_right {
                true => ((self.max_x - max_x) * resolution) as usize,
                false => ((self.min_x - min_x) * resolution) as usize,
            };

            new_back.append(
                &mut (1..x_to_go)
                    .map(|x| (x as f64 / resolution as f64) + min_x)
                    .map(|x| (x, self.run_func(x)))
                    .map(|(x, y)| Value::new(x, y))
                    .collect(),
            );

            self.back_cache = Some(new_back);
        } else {
            self.back_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        }
    }

    pub fn get_step(&self) -> f64 {
        (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64)
    }

    pub fn is_integral(&self) -> bool { self.integral }

    pub fn run(&mut self) -> FunctionOutput {
        let front_values: Vec<Value> = match self.back_cache.is_some() {
            true => {
                // debug_log("back_cache: using");
                self.back_cache.as_ref().expect("").clone()
            }
            false => {
                // debug_log("back_cache: regen");
                let absrange = (self.max_x - self.min_x).abs();
                let resolution: f64 = (self.pixel_width as f64 / absrange) as f64;
                let front_data: Vec<Value> = (1..=self.pixel_width)
                    .map(|x| (x as f64 / resolution as f64) + self.min_x)
                    .map(|x| (x, self.run_func(x)))
                    .map(|(x, y)| Value::new(x, y))
                    .collect();
                // println!("{} {}", front_data.len(), front_data.len() as f64/absrange);

                self.back_cache = Some(front_data.clone());
                front_data
            }
        };

        if self.integral {
            let back_bars: (Vec<Bar>, f64) = match self.front_cache.is_some() {
                true => {
                    // debug_log("front_cache: using");
                    let cache = self.front_cache.as_ref().expect("");
                    let vec_bars: Vec<Bar> = cache.0.to_vec();
                    (vec_bars, cache.1)
                }
                false => {
                    // debug_log("front_cache: regen");
                    let (data, area) = self.integral_rectangles();
                    let bars: Vec<Bar> = data.iter().map(|(x, y)| Bar::new(*x, *y)).collect();

                    let output = (bars, area);
                    self.front_cache = Some(output.clone());
                    output
                }
            };
            FunctionOutput::new(front_values, Some(back_bars))
        } else {
            FunctionOutput::new(front_values, None)
        }
    }

    // Creates and does the math for creating all the rectangles under the graph
    fn integral_rectangles(&self) -> (Vec<(f64, f64)>, f64) {
        if !self.integral {
            panic!("integral_rectangles called, but self.integral is false!");
        }

        if self.integral_min_x.is_nan() {
            panic!("integral_min_x is NaN")
        }

        if self.integral_max_x.is_nan() {
            panic!("integral_max_x is NaN")
        }

        let step = self.get_step();

        let half_step = step / 2.0;
        let data2: Vec<(f64, f64)> = (0..self.integral_num)
            .map(|e| {
                let x: f64 = ((e as f64) * step) + self.integral_min_x;

                // Makes sure rectangles are properly handled on x values below 0
                let x2: f64 = match x > 0.0 {
                    true => x + step,
                    false => x - step,
                };

                let tmp1: f64 = self.run_func(x);
                let tmp2: f64 = self.run_func(x2);

                // Chooses the y value who's absolute value is the smallest
                let mut output = match tmp2.abs() > tmp1.abs() {
                    true => (x, tmp1),
                    false => (x2, tmp2),
                };

                // Applies `half_step` in order to make the bar graph display properly
                if output.0 > 0.0 {
                    output.0 += half_step;
                } else {
                    output.0 -= half_step;
                }

                output
            })
            .filter(|(_, y)| !y.is_nan())
            .collect();
        let area: f64 = data2.iter().map(|(_, y)| y * step).sum(); // sum of all rectangles' areas
        (data2, area)
    }
}
