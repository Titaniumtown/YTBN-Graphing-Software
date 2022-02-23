use core::num;

use crate::chart_manager::ChartManager;
use crate::misc::{add_asterisks, digits_precision, test_func, Cache, Function};
use eframe::{egui, epi};
use egui::widgets::plot::{Bar, BarChart};
use egui::{
    plot::{HLine, Line, Plot, Text, Value, Values},
    Pos2,
};
use egui::{Color32, ColorImage, Ui};
use emath::Rect;
use epaint::{RectShape, Rounding, Stroke};
use meval::Expr;

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
    fn name(&self) -> &str { "eframe template" }

    /// Called once before the first frame.
    fn setup(
        &mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>,
    ) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            func_str,
            min_x,
            max_x,
            num_interval,
            resolution,
            chart_manager,
        } = self;

        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Function: ");
                ui.text_edit_singleline(func_str);
            });
            let func_test_output = test_func(func_str.clone());
            if func_test_output != "" {
                parse_error = func_test_output;
            }

            ui.add(egui::Slider::new(min_x, -1000.0..=1000.0).text("Min X"));
            ui.add(egui::Slider::new(max_x, *min_x..=1000.0).text("Max X"));

            ui.add(egui::Slider::new(num_interval, 0..=usize::MAX).text("Interval"));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if parse_error != "" {
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
                .map(|(_, x2, y)| Bar::new(*x2, *y))
                .collect();
            let barchart = BarChart::new(bars).color(Color32::BLUE);

            ui.label(format!("Area: {}", digits_precision(area, 8)));
            Plot::new("plot")
                .view_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    plot_ui.line(curve);
                    if self.num_interval > 0 {
                        plot_ui.bar_chart(barchart);
                    }
                });
        });
    }
}
