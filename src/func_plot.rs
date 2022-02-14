use crate::DrawResult;
use meval::Expr;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;

pub fn draw(
    element: HtmlCanvasElement, func_str: &str, min_x: f32, max_x: f32, min_y: f32, max_y: f32,
    num_interval: usize, resolution: i32,
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
        .margin(20.0)
        .caption(format!("y={}", func_str), font)
        .x_label_area_size(30.0)
        .y_label_area_size(30.0)
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
    Ok((output, area))
}

// Creates and does the math for creating all the rectangles under the graph
#[inline(always)]
fn integral_rectangles(
    min_x: f32, step: f32, num_interval: usize, func: &dyn Fn(f64) -> f64,
) -> (Vec<(f32, f32, f32)>, f32) {
    let data2: Vec<(f32, f32, f32)> = (0..num_interval)
        .map(|e| {
            let x: f32 = ((e as f32) * step) + min_x;

            let x2: f32 = match x > 0.0 {
                true => x + step,
                false => x - step,
            };

            let tmp1: f32 = func(x as f64) as f32;
            let tmp2: f32 = func(x2 as f64) as f32;

            let y: f32 = match tmp2.abs() > tmp1.abs() {
                true => tmp1,
                false => tmp2,
            };

            if !y.is_nan() {
                (x, x2, y)
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .filter(|ele| ele != &(0.0, 0.0, 0.0))
        .collect();
    let area: f32 = data2.iter().map(|(_, _, y)| y * step).sum(); // sum of all rectangles' areas
    (data2, area)
}
