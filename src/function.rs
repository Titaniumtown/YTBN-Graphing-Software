use meval::Expr;
use egui::plot::Value;
use egui::widgets::plot::Bar;

const RESOLUTION: f64 = 1000.0;


// Struct that stores and manages the output of a function
pub struct FunctionOutput {
    // The actual line graph
    back: Vec<Value>,

    // Integral information
    front: Option<(Vec<Bar>, f64)>,
}

impl FunctionOutput {
    #[inline]
    pub fn new(back: Vec<Value>, front: Option<(Vec<Bar>, f64)>) -> Self {
        Self {
            back,
            front,
        }
    }

    #[inline]
    fn has_integral(&self) -> bool {
        match self.front {
            Some(x) => true,
            None => false,
        }
    }
}

pub struct Function {
    function: Box<dyn Fn(f64) -> f64>,
    func_str: String,
    min_x: f64,
    max_x: f64,
    back_cache: Cache<Vec<Value>>,
    front_cache: Cache<(Vec<Bar>, f64)>,

    integral: bool,
    integral_min_x: f64,
    integral_max_x: f64,
    integral_num: usize,
}

impl Function {
    pub fn new(func_str: String, min_x: f64, max_x: f64, integral: bool, integral_min_x: Option<f64>, integral_max_x: Option<f64>, integral_num: Option<usize>) -> Self {

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
            back_cache: Cache::new_empty(),
            front_cache: Cache::new_empty(),
            integral,
            integral_min_x: match integral_min_x {
                Some(x) => x,
                None => f64::NAN,
            },
            integral_max_x: match integral_max_x {
                Some(x) => x,
                None => f64::NAN,
            },
            integral_num: match integral_num {
                Some(x) => x,
                None => f64::NAN,
            },
        }
    }

    // Runs the internal function to get values
    #[inline]
    fn run_func(&self, x: f64) -> f64 { (self.function)(x) }

    #[inline]
    pub fn update(&mut self, func_str: String, min_x: f64, max_x: f64, integral: bool, integral_min_x: Option<f64>, integral_max_x: Option<f64>, integral_num: Option<usize>) {

        // If the function string changes, just wipe and restart from scratch
        if func_str != self.func_str {
            *self = Self::new(func_str, min_x, max_x, integral, integral_min_x, integral_max_x, integral_num);
            return;
        }


        if (min_x != self.min_x) | (max_x != self.max_x) {
            self.back_cache.invalidate();
            self.min_x = min_x;
            self.max_x = max_x;
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

            if (integral_min_x != Some(self.integral_min_x)) | (integral_max_x != Some(self.integral_max_x)) | (integral_num != Some(self.integral_num)) {
                self.front_cache.invalidate();
                self.integral_min_x = integral_min_x.expect("");
                self.integral_max_x = integral_max_x.expect("");
                self.integral_num = integral_num.expect("");
            }
        }

    }


    #[inline]
    pub fn run(&mut self) -> FunctionOutput {
        let absrange = (self.max_x - self.min_x).abs();
        let output: Vec<(f64, f64)> = (1..=self.resolution)
            .map(|x| ((x as f64 / RESOLUTION) * absrange) + self.min_x_back)
            .map(|x| (x, self.function.run(x)))
            .collect();
        output
    }

    #[inline]
    pub fn str_compare(&self, other_string: String) -> bool { self.func_str == other_string }

    #[inline]
    pub fn get_step(&self) -> f64 { (self.integral_min_x - self.integral_max_x).abs() / (self.num_interval as f64) }

    // Creates and does the math for creating all the rectangles under the graph
    #[inline]
    fn integral_rectangles(&self, step: f64) -> (Vec<(f64, f64)>, f64) {
        let half_step = step / 2.0;
        let data2: Vec<(f64, f64)> = (0..self.num_interval)
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