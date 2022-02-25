use std::ops::RangeInclusive;

use crate::function::Function;
use crate::misc::{digits_precision, test_func};
use eframe::{egui, epi};
use egui::plot::{Line, Plot, Values};
use egui::widgets::plot::BarChart;
use egui::widgets::Button;
use egui::Color32;
use git_version::git_version;

// Grabs git version on compile time
const GIT_VERSION: &str = git_version!();

// Sets some hard-coded limits to the application
const INTEGRAL_NUM_RANGE: RangeInclusive<usize> = 10..=1000000;
const MIN_X_TOTAL: f64 = -1000.0;
const MAX_X_TOTAL: f64 = 1000.0;
const X_RANGE: RangeInclusive<f64> = MIN_X_TOTAL..=MAX_X_TOTAL;
const DEFAULT_FUNCION: &str = "x^2";

pub struct MathApp {
    functions: Vec<Function>,
    min_x: f64,
    max_x: f64,

    // Currently really unused. But once fully implemented it will represent the full graph's min_x and max_x, being seperate from min_x and max_x for the intergral.
    integral_min_x: f64,
    integral_max_x: f64,

    integral_num: usize,
}

impl Default for MathApp {
    #[inline]
    fn default() -> Self {
        let def_min_x = -10.0;
        let def_max_x = 10.0;
        let def_interval: usize = 1000;

        let def_funcs: Vec<Function> = vec![Function::new(
            String::from(DEFAULT_FUNCION),
            def_min_x,
            def_max_x,
            true,
            Some(def_min_x),
            Some(def_max_x),
            Some(def_interval),
        )];

        Self {
            functions: def_funcs,
            min_x: def_min_x,
            max_x: def_max_x,
            integral_min_x: def_min_x,
            integral_max_x: def_max_x,
            integral_num: def_interval,
        }
    }
}

impl MathApp {
    #[inline]
    pub fn get_step(&self) -> f64 {
        (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64)
    }
}

impl epi::App for MathApp {
    // The name of the program (displayed when running natively as the window title)
    fn name(&self) -> &str { "Integral Demonstration" }

    // Called once before the first frame.
    #[inline]
    fn setup(
        &mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>,
    ) {
    }

    // Called each time the UI needs repainting, which may be many times per second.
    #[inline]
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            functions,
            min_x,
            max_x,
            integral_min_x,
            integral_max_x,
            integral_num,
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

        let mut new_func_data: Vec<(String, bool, bool)> = Vec::new();
        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Side Panel");
                if ui.add(egui::Button::new("Add function")).clicked() {
                    functions.push(Function::new(String::from(DEFAULT_FUNCION), *min_x,
                        *max_x,
                        true,
                        Some(*min_x),
                        Some(*max_x),
                        Some(*integral_num)));
                }

                for function in functions.iter() {
                    let mut func_str = function.get_string();
                    let mut integral_toggle: bool = false;
                    ui.horizontal(|ui| {
                        ui.label("Function: ");
                        if ui.add(Button::new("Toggle Integrals")).clicked() {
                            integral_toggle = true;
                        }
                        ui.text_edit_singleline(&mut func_str);
                    });

                    let func_test_output = test_func(func_str.clone());
                    let mut got_error: bool = false;
                    if !func_test_output.is_empty() {
                        parse_error += &func_test_output;
                        got_error = true;
                    }

                    new_func_data.push((func_str, integral_toggle, got_error));
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
                    *integral_min_x = *min_x;
                    *integral_max_x = *max_x;
                }

                ui.add(egui::Slider::new(integral_num, INTEGRAL_NUM_RANGE).text("Interval"));

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

                let mut i: usize = 0;
                for function in functions.iter_mut() {
                    let (func_str, integral_toggle, got_error) = (new_func_data[i].0.clone(), new_func_data[i].1, new_func_data[i].2);

                    let integral: bool = if integral_toggle {
                        !function.is_integral()
                    } else {
                        function.is_integral()
                    };


                    function.update(func_str, *min_x, *max_x, integral, Some(*integral_min_x), Some(*integral_max_x), Some(*integral_num), got_error);
                    i += 1;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }

            let step = self.get_step();
            let mut area_list: Vec<f64> = Vec::new();
            Plot::new("plot")
                .view_aspect(1.0)
                .data_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    for function in self.functions.iter_mut() {
                        if function.is_broken() {
                            continue;
                        }

                        let output = function.run();
                        let back = output.get_back();
                        plot_ui.line(Line::new(Values::from_values(back)).color(Color32::RED));

                        if output.has_integral() {
                            let (bars, area) = output.get_front();
                            let bar_chart =
                                BarChart::new(bars.clone()).color(Color32::BLUE).width(step);
                            plot_ui.bar_chart(bar_chart);
                            area_list.push(digits_precision(area, 8))
                        } else {
                            area_list.push(0.0);
                        }
                    }
                });

            let duration = start.elapsed();

            ui.label(format!(
                "Area: {:?} Took: {:?}",
                area_list.clone(),
                duration
            ));
        });
    }
}
