use crate::chart_manager::ChartManager;
use crate::misc::{digits_precision, test_func, Cache};
use eframe::{egui, epi};
use egui::plot::{Line, Plot, Value, Values};
use egui::widgets::plot::{Bar, BarChart};
use egui::Color32;

pub struct MathApp {
    func_str: String,
    min_x: f64,
    max_x: f64,
    num_interval: usize,
    resolution: usize,
    chart_manager: ChartManager,
    bar_cache: Cache<Vec<Bar>>,
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
            bar_cache: Cache::new_empty(),
        }
    }
}

impl epi::App for MathApp {
    fn name(&self) -> &str { "Integral Demonstration" }

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
            resolution,
            chart_manager,
            bar_cache,
        } = self;

        // Note: This Instant implementation does not show microseconds when using wasm.
        let start = instant::Instant::now();

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
            let min_x_old = *min_x;
            let min_x_response = ui.add(egui::Slider::new(min_x, x_range.clone()).text("Min X"));

            let max_x_old = *max_x;
            let max_x_response = ui.add(egui::Slider::new(max_x, x_range).text("Max X"));

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

            ui.add(egui::Slider::new(num_interval, 10..=usize::MAX).text("Interval"));

            ui.hyperlink_to(
                "I'm Opensource! (and licensed under AGPLv3)",
                "https://github.com/Titaniumtown/integral_site",
            );
        });

        // let update_back = chart_manager.do_update_back(func_str.clone(), *min_x, *max_x);
        let update_front = chart_manager.do_update_front(*num_interval, *resolution);

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }

            let (filtered_data, rect_data, area) = chart_manager.update(
                self.func_str.clone(),
                *min_x,
                *max_x,
                *num_interval,
                *resolution,
            );

            let filtered_data_values = filtered_data
                .iter()
                .map(|(x, y)| Value::new(*x, *y))
                .collect();

            let curve = Line::new(Values::from_values(filtered_data_values)).color(Color32::RED);

            let bars: Vec<Bar> = match update_front {
                true => {
                    let bars: Vec<Bar> = rect_data.iter().map(|(x, y)| Bar::new(*x, *y)).collect();

                    bar_cache.set(bars.clone());
                    bars
                }
                false => {
                    if bar_cache.is_valid() {
                        bar_cache.get().clone()
                    } else {
                        let bars: Vec<Bar> =
                            rect_data.iter().map(|(x, y)| Bar::new(*x, *y)).collect();

                        bar_cache.set(bars.clone());
                        bars
                    }
                }
            };
            let bar_chart = BarChart::new(bars)
                .color(Color32::BLUE)
                .width(chart_manager.get_step());

            Plot::new("plot")
                .view_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    plot_ui.line(curve);
                    plot_ui.bar_chart(bar_chart);
                });

            let duration = start.elapsed();

            ui.label(format!(
                "Area: {} Took: {:?}",
                digits_precision(area, 8),
                duration
            ));
        });

        // Cute little window that lists supported function!
        // TODO: add more detail
        egui::Window::new("Supported Functions").show(ctx, |ui| {
            ui.label("- sqrt, abs");
            ui.label("- exp, ln, log10 (log10 can also be called as log)");
            ui.label("- sin, cos, tan, asin, acos, atan, atan2");
            ui.label("- sinh, cosh, tanh, asinh, acosh, atanh");
            ui.label("- floor, ceil, round");
            ui.label("- signum, min, max");
        });
    }
}
