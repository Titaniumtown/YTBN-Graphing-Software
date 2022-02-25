use crate::misc::{add_asterisks, Function};

pub enum UpdateType {
    Full,
    Front,
    Back,
    None,
}

// Manages Chart generation and caching of values
pub struct ChartManager {
    function: Function,
    min_x: f64,
    max_x: f64,
    min_x_back: f64,
    max_x_back: f64,
    num_interval: usize,
    resolution: usize,
}

impl ChartManager {
    pub fn new(
        func_str: String, min_x: f64, max_x: f64, min_x_back: f64, max_x_back: f64,
        num_interval: usize, resolution: usize,
    ) -> Self {
        Self {
            function: Function::from_string(func_str),
            min_x,
            max_x,
            min_x_back,
            max_x_back,
            num_interval,
            resolution,
        }
    }

    #[inline]
    pub fn draw_back(&mut self) -> Vec<(f64, f64)> {
        let absrange = (self.max_x_back - self.min_x_back).abs();
        let output: Vec<(f64, f64)> = (1..=self.resolution)
            .map(|x| ((x as f64 / self.resolution as f64) * absrange) + self.min_x_back)
            .map(|x| (x, self.function.run(x)))
            .collect();
        output
    }

    #[inline]
    pub fn draw_front(&mut self) -> (Vec<(f64, f64)>, f64) {
        self.integral_rectangles(self.get_step())
    }

    #[inline]
    pub fn get_step(&self) -> f64 { (self.max_x - self.min_x).abs() / (self.num_interval as f64) }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self, func_str_new: String, min_x: f64, max_x: f64, min_x_back: f64, max_x_back: f64,
        num_interval: usize, resolution: usize,
    ) -> UpdateType {
        let func_str: String = add_asterisks(func_str_new);
        let update_func: bool = !self.function.str_compare(func_str.clone());

        let update_back =
            update_func | (min_x_back != self.min_x_back) | (max_x_back != self.max_x_back);
        let update_front = update_func
            | (min_x != self.min_x)
            | (max_x != self.max_x)
            | (self.resolution != resolution)
            | (num_interval != self.num_interval);

        if update_func {
            self.function = Function::from_string(func_str);
        }

        self.min_x = min_x;
        self.max_x = max_x;
        self.min_x_back = min_x_back;
        self.max_x_back = max_x_back;
        self.num_interval = num_interval;
        self.resolution = resolution;

        if update_back && update_front {
            UpdateType::Full
        } else if update_back {
            UpdateType::Back
        } else if update_front {
            UpdateType::Front
        } else {
            UpdateType::None
        }
    }

    // Creates and does the math for creating all the rectangles under the graph
    #[inline]
    fn integral_rectangles(&self, step: f64) -> (Vec<(f64, f64)>, f64) {
        let half_step = step / 2.0;
        let data2: Vec<(f64, f64)> = (0..self.num_interval)
            .map(|e| {
                let x: f64 = ((e as f64) * step) + self.min_x;

                // Makes sure rectangles are properly handled on x values below 0
                let x2: f64 = match x > 0.0 {
                    true => x + step,
                    false => x - step,
                };

                let tmp1: f64 = self.function.run_func(x);
                let tmp2: f64 = self.function.run_func(x2);

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
