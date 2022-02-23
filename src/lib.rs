#![allow(clippy::unused_unit)] // Fixes clippy keep complaining about wasm_bindgen
#![allow(clippy::type_complexity)] // Clippy, my types are fine.

use meval::Expr;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
mod misc;
use crate::misc::{add_asterisks, Cache, ChartOutput, DrawResult, Function};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Manages Chart generation and caching of values
#[wasm_bindgen]
pub struct ChartManager {
    function: Function,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    num_interval: usize,
    resolution: usize,
    back_cache: Cache<Vec<(f32, f32)>>,
    front_cache: Cache<(Vec<(f32, f32, f32)>, f32)>,
}

#[wasm_bindgen]
impl ChartManager {
    pub fn new(
        func_str: String, min_x: f32, max_x: f32, min_y: f32, max_y: f32, num_interval: usize,
        resolution: usize,
    ) -> Self {
        Self {
            function: Function::from_string(func_str),
            min_x,
            max_x,
            min_y,
            max_y,
            num_interval,
            resolution,
            back_cache: Cache::new_empty(),
            front_cache: Cache::new_empty(),
        }
    }

    // Used in order to hook into `panic!()` to log in the browser's console
    pub fn init_panic_hook() { panic::set_hook(Box::new(console_error_panic_hook::hook)); }

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

    #[inline]
    fn draw(
        &mut self, element: HtmlCanvasElement, dark_mode: bool,
    ) -> DrawResult<(impl Fn((i32, i32)) -> Option<(f32, f32)>, f32)> {
        let backend = CanvasBackend::with_canvas_object(element).unwrap();
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        if dark_mode {
            root.fill(&RGBColor(28, 28, 28))?;
        } else {
            root.fill(&WHITE)?;
        }

        let mut chart = ChartBuilder::on(&root)
            .margin(20.0)
            .caption(format!("y={}", self.function.get_string()), font)
            .x_label_area_size(30.0)
            .y_label_area_size(30.0)
            .build_cartesian_2d(self.min_x..self.max_x, self.min_y..self.max_y)?;

        if dark_mode {
            chart
                .configure_mesh()
                .x_labels(3)
                .y_labels(3)
                .light_line_style(&RGBColor(254, 254, 254))
                .draw()?;
        } else {
            chart.configure_mesh().x_labels(3).y_labels(3).draw()?;
        }

        let absrange = (self.max_x - self.min_x).abs();
        let data: Vec<(f32, f32)> = match self.back_cache.is_valid() {
            true => self.back_cache.get().clone(),
            false => {
                log("Updating back_cache");
                let output: Vec<(f32, f32)> = (1..=self.resolution)
                    .map(|x| ((x as f32 / self.resolution as f32) * absrange) + self.min_x)
                    .map(|x| (x, self.function.run(x)))
                    .collect();
                self.back_cache.set(output.clone());
                output
            }
        };

        let filtered_data: Vec<(f32, f32)> = data
            .iter()
            .filter(|(_, y)| &self.min_y <= y && y <= &self.max_y)
            .map(|(x, y)| (*x, *y))
            .collect();
        chart.draw_series(LineSeries::new(filtered_data, &RED))?;

        let (rect_data, area): (Vec<(f32, f32, f32)>, f32) = match self.front_cache.is_valid() {
            true => self.front_cache.get().clone(),
            false => {
                log("Updating front_cache");
                let step = absrange / (self.num_interval as f32);
                let output: (Vec<(f32, f32, f32)>, f32) = self.integral_rectangles(step);
                self.front_cache.set(output.clone());
                output
            }
        };

        if self.num_interval <= 200 {
            // Draw rectangles
            chart.draw_series(
                rect_data
                    .iter()
                    .map(|(x1, x2, y)| Rectangle::new([(*x2, *y), (*x1, 0.0)], &BLUE)),
            )?;
        } else {
            // Save resources by not graphing rectangles and using an AreaSeries when you can no longer see the rectangles
            let capped_data: Vec<(f32, f32)> = data
                .iter()
                .map(|(x, y)| {
                    if y.is_nan() {
                        return (*x, 0.0);
                    }

                    let new_y: &f32 = if y > &self.max_y {
                        &self.max_y
                    } else if &self.min_y > y {
                        &self.min_y
                    } else {
                        y
                    };

                    (*x, *new_y)
                })
                .collect();
            chart.draw_series(AreaSeries::new(capped_data, 0.0, &BLUE))?;
        }

        root.present()?;
        Ok((chart.into_coord_trans(), area))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self, canvas: HtmlCanvasElement, func_str_new: String, min_x: f32, max_x: f32,
        min_y: f32, max_y: f32, num_interval: usize, resolution: usize, dark_mode: bool,
    ) -> Result<ChartOutput, JsValue> {
        let func_str: String = add_asterisks(func_str_new);
        let update_func: bool = !self.function.str_compare(func_str.clone());

        let underlying_update = update_func
            | (min_x != self.min_x)
            | (max_x != self.max_x)
            | (min_y != self.min_y)
            | (max_y != self.max_y);

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
        self.min_y = min_y;
        self.max_y = max_y;
        self.num_interval = num_interval;
        self.resolution = resolution;

        let draw_output = self
            .draw(canvas, dark_mode)
            .map_err(|err| err.to_string())?;
        let map_coord = draw_output.0;

        let chart_output = ChartOutput {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x, y))),
            area: draw_output.1,
        };

        Ok(chart_output)
    }

    // Creates and does the math for creating all the rectangles under the graph
    #[inline]
    fn integral_rectangles(&self, step: f32) -> (Vec<(f32, f32, f32)>, f32) {
        let data2: Vec<(f32, f32, f32)> = (0..self.num_interval)
            .map(|e| {
                let x: f32 = ((e as f32) * step) + self.min_x;

                // Makes sure rectangles are properly handled on x values below 0
                let x2: f32 = match x > 0.0 {
                    true => x + step,
                    false => x - step,
                };

                let tmp1: f32 = self.function.run(x);
                let tmp2: f32 = self.function.run(x2);

                // Chooses the y value who's absolute value is the smallest
                let y: f32 = match tmp2.abs() > tmp1.abs() {
                    true => tmp1,
                    false => tmp2,
                };

                (x, x2, y)
            })
            .filter(|(_, _, y)| !y.is_nan())
            .collect();
        let area: f32 = data2.iter().map(|(_, _, y)| y * step).sum(); // sum of all rectangles' areas
        (data2, area)
    }
}
