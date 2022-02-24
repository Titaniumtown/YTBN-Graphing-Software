use crate::misc::{add_asterisks, Cache, Function};

// Manages Chart generation and caching of values
pub struct ChartManager {
    function: Function,
    min_x: f64,
    max_x: f64,
    num_interval: usize,
    resolution: usize,
    back_cache: Cache<Vec<(f64, f64)>>,
    front_cache: Cache<(Vec<(f64, f64)>, f64)>,
}

impl ChartManager {
    pub fn new(
        func_str: String, min_x: f64, max_x: f64, num_interval: usize, resolution: usize,
    ) -> Self {
        Self {
            function: Function::from_string(func_str),
            min_x,
            max_x,
            num_interval,
            resolution,
            back_cache: Cache::new_empty(),
            front_cache: Cache::new_empty(),
        }
    }

    #[inline]
    fn draw(&mut self) -> (Vec<(f64, f64)>, Vec<(f64, f64)>, f64) {
        let absrange = (self.max_x - self.min_x).abs();
        let data: Vec<(f64, f64)> = match self.back_cache.is_valid() {
            true => self.back_cache.get().clone(),
            false => {
                let output: Vec<(f64, f64)> = (1..=self.resolution)
                    .map(|x| ((x as f64 / self.resolution as f64) * absrange) + self.min_x)
                    .map(|x| (x, self.function.run(x)))
                    .collect();
                self.back_cache.set(output.clone());
                output
            }
        };

        let filtered_data: Vec<(f64, f64)> = data.iter().map(|(x, y)| (*x, *y)).collect();

        let (rect_data, area): (Vec<(f64, f64)>, f64) = match self.front_cache.is_valid() {
            true => self.front_cache.get().clone(),
            false => {
                let step = absrange / (self.num_interval as f64);
                let output: (Vec<(f64, f64)>, f64) = self.integral_rectangles(step);
                self.front_cache.set(output.clone());
                output
            }
        };

        (filtered_data, rect_data, area)
    }

    pub fn get_step(&self) -> f64 { (self.max_x - self.min_x).abs() / (self.num_interval as f64) }

    pub fn do_update_front(&self, resolution: usize, num_interval: usize) -> bool {
        (self.resolution != resolution) | (num_interval != self.num_interval)
    }

    pub fn do_update_back(&self, func_str_new: String, min_x: f64, max_x: f64) -> bool {
        let func_str: String = add_asterisks(func_str_new);
        let update_func: bool = !self.function.str_compare(func_str);

        update_func | (min_x != self.min_x) | (max_x != self.max_x)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self, func_str_new: String, min_x: f64, max_x: f64, num_interval: usize,
        resolution: usize,
    ) -> (Vec<(f64, f64)>, Vec<(f64, f64)>, f64) {
        let func_str: String = add_asterisks(func_str_new);
        let update_func: bool = !self.function.str_compare(func_str.clone());

        let underlying_update = update_func | (min_x != self.min_x) | (max_x != self.max_x);

        if underlying_update | (self.resolution != resolution) {
            self.back_cache.invalidate();
        }

        if underlying_update | (num_interval != self.num_interval) {
            self.front_cache.invalidate();
        }

        if update_func {
            self.function = Function::from_string(func_str);
        }

        self.min_x = min_x;
        self.max_x = max_x;
        self.num_interval = num_interval;
        self.resolution = resolution;

        self.draw()
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

                let tmp1: f64 = self.function.run(x);
                let tmp2: f64 = self.function.run(x2);

                // Chooses the y value who's absolute value is the smallest
                let mut output = match tmp2.abs() > tmp1.abs() {
                    true => (x, tmp1),
                    false => (x2, tmp2),
                };

                // Applies `half_step` in order to make the bar graph display properly
                if output.0 > 0.0 {
                    output.0 = output.0 + half_step;
                } else {
                    output.0 = output.0 - half_step;
                }

                output
            })
            .filter(|(_, y)| !y.is_nan())
            .collect();
        let area: f64 = data2.iter().map(|(_, y)| y * step).sum(); // sum of all rectangles' areas
        (data2, area)
    }
}
