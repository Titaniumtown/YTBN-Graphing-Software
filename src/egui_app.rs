use crate::chart_manager::ChartManager;
use crate::misc::{digits_precision, test_func};
use eframe::{egui, epi};
use egui::widgets::plot::{Bar, BarChart};
use egui::{
    plot::{Line, Plot, Value, Values},
};
use egui::{Color32};
use std::time::Instant;

pub struct MathApp {
    func_str: String,
    min_x: f64,
    max_x: f64,
    num_interval: usize,
    resolution: usize,
    chart_manager: ChartManager,
}

impl Default for MathApp {
    fn default() -> Self {
        Self {
            func_str: "x^2".to_string(),
            min_x: -10.0,
            max_x: 10.0,
            num_interval: 100,
            resolution: 10000,
            chart_manager: ChartManager::new("x^2".to_string(), -10.0, 10.0, 100, 10000),
        }
    }
}

impl epi::App for MathApp {
    fn name(&self) -> &str { "Integral Demo" }

    /// Called once before the first frame.
    fn setup(
        &mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>,
    ) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            func_str,
            min_x,
            max_x,
            num_interval,
            resolution: _,
            chart_manager,
        } = self;
        let start = Instant::now();

        let min_x_total: f32 = -1000.0;
        let max_x_total: f32 = 1000.0;

        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Function: ");
                ui.text_edit_singleline(func_str);
            });
            let func_test_output = test_func(func_str.clone());
            if !func_test_output.is_empty() {
                parse_error = func_test_output;
            }

            let x_range = min_x_total as f64..=max_x_total as f64;
            let min_x_old = min_x.clone();
            let min_x_response = ui.add(egui::Slider::new(min_x, x_range.clone()).text("Min X"));

            let max_x_old = max_x.clone();
            let max_x_response =  ui.add(egui::Slider::new(max_x, x_range).text("Max X"));

            if min_x >= max_x {
                if max_x_response.changed() {
                    *max_x = max_x_old;
                } else if min_x_response.changed() {
                    *min_x = min_x_old;
                } else {
                    *min_x = -10.0;
                    *max_x = 10.0;
                }
            }



            ui.add(egui::Slider::new(num_interval, 1..=usize::MAX).text("Interval"));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }

            let (filtered_data, rect_data, area) = chart_manager.update(
                self.func_str.clone(),
                self.min_x,
                self.max_x,
                self.num_interval,
                self.resolution,
            );

            let filtered_data_values = filtered_data
                .iter()
                .map(|(x, y)| Value::new(*x, *y))
                .collect();

            let curve = Line::new(Values::from_values(filtered_data_values)).color(Color32::RED);

            let bars = rect_data
                .iter()
                .map(|(x, y)| Bar::new(*x, *y))
                .collect();
            let barchart = BarChart::new(bars).color(Color32::BLUE);
            Plot::new("plot")
                .view_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    plot_ui.line(curve);
                    plot_ui.bar_chart(barchart);
                });

            let duration = start.elapsed();

            ui.label(format!("Area: {} Took: {:?}", digits_precision(area, 8), duration));
        });
    }
}
