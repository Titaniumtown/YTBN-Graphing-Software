use std::ops::RangeInclusive;

use crate::function::Function;
use crate::misc::{digits_precision, test_func};
use eframe::{egui, epi};
use egui::plot::{Line, Plot, Values};
use egui::widgets::plot::BarChart;
use egui::widgets::Button;
use egui::{Color32, Vec2};
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

    // No clue why I need this, but I do. Rust being weird I guess.
    // Ideally this should be information directly accessed from `functions` but it always returns an empty string. I don't know, I've been debuging this for a while now.
    func_strs: Vec<String>,

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

        Self {
            functions: vec![Function::new(
                String::from(DEFAULT_FUNCION),
                def_min_x,
                def_max_x,
                true,
                Some(def_min_x),
                Some(def_max_x),
                Some(def_interval),
            )],
            func_strs: vec![String::from(DEFAULT_FUNCION)],
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
    #[inline(always)]
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            functions,
            func_strs,
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

        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Side Panel");
                if ui.add(egui::Button::new("Add function")).clicked() {
                    // min_x and max_x will be updated later, doesn't matter here
                    functions.push(Function::new(String::from(DEFAULT_FUNCION), -1.0,
                        1.0,
                        false,
                        None,
                        None,
                        None));
                    func_strs.push(String::from(DEFAULT_FUNCION));
                }

                let min_x_old = *integral_min_x;
                let min_x_response =
                    ui.add(egui::Slider::new(integral_min_x, X_RANGE.clone()).text("Min X"));

                let max_x_old = *integral_max_x;
                let max_x_response = ui.add(egui::Slider::new(integral_max_x, X_RANGE).text("Max X"));

                // Checks bounds, and if they are invalid, fix them
                if integral_min_x >= integral_max_x {
                    if max_x_response.changed() {
                        *integral_max_x = max_x_old;
                    } else if min_x_response.changed() {
                        *integral_min_x = min_x_old;
                    } else {
                        *integral_min_x = -10.0;
                        *integral_max_x = 10.0;
                    }
                }

                ui.add(egui::Slider::new(integral_num, INTEGRAL_NUM_RANGE).text("Interval"));

                for (i, function) in functions.iter_mut().enumerate() {
                    let mut integral_toggle: bool = false;
                    ui.horizontal(|ui| {
                        ui.label("Function: ");
                        if ui.add(Button::new("Toggle Integrals")).clicked() {
                            integral_toggle = true;
                        }
                        ui.text_edit_singleline(&mut func_strs[i]);
                    });

                    let integral: bool = if integral_toggle {
                        !function.is_integral()
                    } else {
                        function.is_integral()
                    };

                    if func_strs[i] != "" {
                        let func_test_output = test_func(func_strs[i].clone());
                        if !func_test_output.is_empty() {
                            parse_error += &func_test_output;
                        } else {
                            function.update(func_strs[i].clone(), integral, Some(*integral_min_x), Some(*integral_max_x), Some(*integral_num));
                        }
                    }
                }

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

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }

            let step = self.get_step();
            let mut area_list: Vec<f64> = Vec::new();
            Plot::new("plot")
                .set_margin_fraction(Vec2::ZERO)
                .view_aspect(1.0)
                .data_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    let bounds = plot_ui.plot_bounds();
                    let minx_bounds: f64 = bounds.min()[0];
                    let maxx_bounds: f64 = bounds.max()[0];
                    // println!("({}, {})", minx_bounds, maxx_bounds);

                    for (i, function) in self.functions.iter_mut().enumerate() {
                        function.update_bounds(minx_bounds, maxx_bounds);
                        if self.func_strs[i].is_empty() {
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
