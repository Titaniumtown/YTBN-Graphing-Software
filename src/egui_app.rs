use std::ops::RangeInclusive;

use crate::chart_manager::{ChartManager, UpdateType};
use crate::misc::{digits_precision, test_func, Cache};
use eframe::{egui, epi};
use egui::plot::{Line, Plot, Value, Values};
use egui::widgets::plot::{Bar, BarChart};
use egui::Color32;
use git_version::git_version;

// Grabs git version on compile time
const GIT_VERSION: &str = git_version!();

// Sets some hard-coded limits to the application
const NUM_INTERVAL_RANGE: RangeInclusive<usize> = 10..=10000;
const MIN_X_TOTAL: f64 = -1000.0;
const MAX_X_TOTAL: f64 = 1000.0;
const X_RANGE: RangeInclusive<f64> = MIN_X_TOTAL..=MAX_X_TOTAL;

pub struct MathApp {
    func_str: String,
    min_x: f64,
    max_x: f64,

    // Currently really unused. But once fully implemented it will represent the full graph's min_x and max_x, being seperate from min_x and max_x for the intergral.
    min_x_graph: f64,
    max_x_graph: f64,

    num_interval: usize,
    resolution: usize,
    chart_manager: ChartManager,
    back_cache: Cache<Vec<Value>>,
    front_cache: Cache<(Vec<Bar>, f64)>,
}

impl Default for MathApp {
    fn default() -> Self {
        let def_func = "x^2".to_string();
        let def_min_x = -10.0;
        let def_max_x = 10.0;
        let def_interval: usize = 100;
        let def_resolution: usize = 10000;

        Self {
            func_str: def_func.clone(),
            min_x: def_min_x,
            max_x: def_max_x,
            min_x_graph: def_min_x,
            max_x_graph: def_max_x,
            num_interval: def_interval,
            resolution: def_resolution,
            chart_manager: ChartManager::new(
                def_func,
                def_min_x,
                def_max_x,
                def_min_x,
                def_max_x,
                def_interval,
                def_resolution,
            ),
            back_cache: Cache::new_empty(),
            front_cache: Cache::new_empty(),
        }
    }
}

impl MathApp {
    #[inline]
    fn get_back(&mut self) -> Line {
        let data = if self.back_cache.is_valid() {
            self.back_cache.get().clone()
        } else {
            let data = self.chart_manager.draw_back();
            let data_values: Vec<Value> = data.iter().map(|(x, y)| Value::new(*x, *y)).collect();
            self.back_cache.set(data_values.clone());
            data_values
        };
        Line::new(Values::from_values(data)).color(Color32::RED)
    }

    #[inline]
    fn get_front(&mut self) -> (Vec<Bar>, f64) {
        if self.front_cache.is_valid() {
            let cache = self.front_cache.get();
            let vec_bars: Vec<Bar> = cache.0.to_vec();
            (vec_bars, cache.1)
        } else {
            let (data, area) = self.chart_manager.draw_front();
            let bars: Vec<Bar> = data.iter().map(|(x, y)| Bar::new(*x, *y)).collect();

            let output = (bars, area);
            self.front_cache.set(output.clone());
            output
        }
    }

    #[inline]
    fn get_data(&mut self) -> (Line, Vec<Bar>, f64) {
        let (bars, area) = self.get_front();
        (self.get_back(), bars, area)
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
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            func_str,
            min_x,
            max_x,
            min_x_graph,
            max_x_graph,
            num_interval,
            resolution,
            chart_manager,
            back_cache,
            front_cache,
        } = self;

        // Note: This Instant implementation does not show microseconds when using wasm.
        let start = instant::Instant::now();

        // Cute little window that lists supported functions!
        // TODO: add more detail
        egui::Window::new("Supported Functions")
            .default_pos([200.0, 200.0])
            .show(ctx, |ui| {
                ui.label("- sqrt, abs");
                ui.label("- exp, ln, log10 (log10 can also be called as log)");
                ui.label("- sin, cos, tan, asin, acos, atan, atan2");
                ui.label("- sinh, cosh, tanh, asinh, acosh, atanh");
                ui.label("- floor, ceil, round");
                ui.label("- signum, min, max");
            });

        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Side Panel");

                ui.horizontal(|ui| {
                    ui.label("Function: ");
                    ui.text_edit_singleline(func_str);
                });

                let func_test_output = test_func(func_str.clone());
                if !func_test_output.is_empty() {
                    parse_error = func_test_output;
                }
                let min_x_old = *min_x;
                let min_x_response =
                    ui.add(egui::Slider::new(min_x, X_RANGE.clone()).text("Min X"));

                let max_x_old = *max_x;
                let max_x_response = ui.add(egui::Slider::new(max_x, X_RANGE).text("Max X"));

                // Checks bounds, and if they are invalid, fix them
                if min_x >= max_x {
                    if max_x_response.changed() {
                        *max_x = max_x_old;
                    } else if min_x_response.changed() {
                        *min_x = min_x_old;
                    } else {
                        *min_x = -10.0;
                        *max_x = 10.0;
                    }
                    *min_x_graph = *min_x;
                    *max_x_graph = *max_x;
                }

                ui.add(egui::Slider::new(num_interval, NUM_INTERVAL_RANGE).text("Interval"));

                // Opensource and Licensing information
                ui.horizontal(|ui| {
                    ui.hyperlink_to(
                        "I'm Opensource!",
                        "https://github.com/Titaniumtown/integral_site",
                    );
                    ui.label("(and licensed under AGPLv3)").on_hover_text("The AGPL license ensures that the end user, even if not hosting the program itself, still is guaranteed access to the source code of the project in question.");
                });

                // Displays commit info
                ui.horizontal(|ui| {
                    ui.label("Commit: ");

                    // Only include hyperlink if the build doesn't have untracked files
                    if !GIT_VERSION.contains("-modified") {
                        ui.hyperlink_to(
                            GIT_VERSION,
                            format!(
                                "https://github.com/Titaniumtown/integral_site/commit/{}",
                                GIT_VERSION
                            ),
                        );
                    } else {
                        ui.label(GIT_VERSION);
                    }
                });
            });

        if parse_error.is_empty() {
            let do_update = chart_manager.update(
                func_str.clone(),
                *min_x,
                *max_x,
                *min_x_graph,
                *max_x_graph,
                *num_interval,
                *resolution,
            );

            // Invalidates caches according to what settings were changed
            match do_update {
                UpdateType::Full => {
                    back_cache.invalidate();
                    front_cache.invalidate();
                }
                UpdateType::Back => back_cache.invalidate(),
                UpdateType::Front => front_cache.invalidate(),
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }

            let (curve, bars, area) = self.get_data();

            let bar_chart = BarChart::new(bars)
                .color(Color32::BLUE)
                .width(self.chart_manager.get_step());

            Plot::new("plot")
                .view_aspect(1.0)
                .data_aspect(1.0)
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
    }
}
