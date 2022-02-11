use crate::DrawResult;
use meval::Expr;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;

pub fn draw(
    element: HtmlCanvasElement,
    func_str: &str,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    num_interval: usize,
    resolution: i32,
) -> DrawResult<(impl Fn((i32, i32)) -> Option<(f32, f32)>, f32)> {
    let expr: Expr = func_str.parse().unwrap();
    let func = expr.bind("x").unwrap();

    let absrange = (max_x - min_x).abs();
    let step = absrange / (num_interval as f32);
    let backend = CanvasBackend::with_canvas_object(element).unwrap();

    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20 as f32)
        .caption(format!("y={}", func_str), font)
        .x_label_area_size(30 as f32)
        .y_label_area_size(30 as f32)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw()?;

    let data: Vec<(f32, f32)> = (1..=resolution)
        .map(|x| ((x as f32 / resolution as f32) * (&absrange)) + &min_x)
        .map(|x| (x, func(x as f64) as f32))
        .filter(|(_, y)| &min_y <= y && y <= &max_y)
        .collect();

    chart.draw_series(LineSeries::new(data, &RED))?;

    let (data2, area): (Vec<(f32, f32, f32)>, f32) =
        integral_rectangles(min_x, step, num_interval, &func); // Get rectangle coordinates and the total area

    // Draw rectangles
    chart.draw_series(
        data2
            .iter()
            .map(|(x1, x2, y)| Rectangle::new([(*x2, *y), (*x1, 0.0)], &BLUE)),
    )?;

    root.present()?;
    let output = chart.into_coord_trans();
    return Ok((output, area));
}

// Creates and does the math for creating all the rectangles under the graph
#[inline(always)]
fn integral_rectangles(
    min_x: f32,
    step: f32,
    num_interval: usize,
    func: &dyn Fn(f64) -> f64,
) -> (Vec<(f32, f32, f32)>, f32) {
    let mut area: f32 = 0.0; // sum of all rectangles' areas
    let mut tmp1: f32; // Top left Y value that's tested
    let mut tmp2: f32; // Top right Y value that's tested
    let mut x2: f32; // X value of the right side of the rectangle
    let mut y: f32; // Y value of the top of the rectangle
    let mut x: f32; // X value of the left side of the rectangle
    let mut data2: Vec<(f32, f32, f32)> = Vec::new();
    for e in 0..num_interval {
        x = ((e as f32) * step) + min_x;

        if x > 0.0 {
            x2 = x + step;
        } else {
            x2 = x - step;
        }
        tmp1 = func(x as f64) as f32;
        tmp2 = func(x2 as f64) as f32;

        if tmp2.abs() > tmp1.abs() {
            y = tmp1;
        } else {
            y = tmp2;
        }

        // Add current rectangle's area to the total
        if !y.is_nan() {
            area += y * step;
            data2.push((x, x2, y));
        }
    }
    return (data2, area);
}
