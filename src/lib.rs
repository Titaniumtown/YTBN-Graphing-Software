use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
mod func_plot;
use meval::Expr;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

#[wasm_bindgen]
pub struct Chart {
    convert: Box<dyn Fn((i32, i32)) -> Option<(f32, f32)>>,
    area: f32
}

/// Result of screen to chart coordinates conversion.
#[wasm_bindgen]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[wasm_bindgen]
impl Chart {
    pub fn draw(canvas: HtmlCanvasElement, func: &str, minX: f32, maxX: f32, minY: f32, maxY: f32, num_interval: usize, resolution: i32) -> Result<Chart, JsValue> {
        let output = func_plot::draw(canvas, func, minX, maxX, minY, maxY, num_interval, resolution).map_err(|err| err.to_string())?;
        let map_coord = output.0;
        Ok(Chart {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x.into(), y.into()))),
            area: output.1,
        })
    }

    pub fn get_area(&self) -> Result<f32, JsValue> {
        return Ok(self.area);
    }

    // Does actual calculation of antiderivative
    // pub fn actual_area(power: f32, minX: f32, maxX: f32) -> Result<f32, JsValue> {
    //     let newpower = power + 1.0;
    //     return Ok((maxX.powf(newpower as f32)/newpower) - (minX.powf(newpower as f32)/newpower));
    // }

    pub fn coord(&self, x: i32, y: i32) -> Option<Point> {
        (self.convert)((x, y)).map(|(x, y)| Point { x, y })
    }
}
