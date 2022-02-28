use std::ops::RangeInclusive;

use crate::function::Function;
use crate::misc::{add_asterisks, digits_precision, test_func};
use eframe::{egui, epi};
use egui::plot::{Line, Plot, Values};
use egui::widgets::plot::BarChart;
use egui::widgets::Button;
use egui::{Color32, Vec2, FontData, FontFamily};
use git_version::git_version;

// Grabs git version on compile time
const GIT_VERSION: &str = git_version!();

// Sets some hard-coded limits to the application
const INTEGRAL_NUM_RANGE: RangeInclusive<usize> = 10..=100000;
const MIN_X_TOTAL: f64 = -1000.0;
const MAX_X_TOTAL: f64 = 1000.0;
const X_RANGE: RangeInclusive<f64> = MIN_X_TOTAL..=MAX_X_TOTAL;
const DEFAULT_FUNCION: &str = "x^2"; // Default function that appears when adding a new function

pub struct MathApp {
    functions: Vec<Function>,

    // No clue why I need this, but I do. Rust being weird I guess.
    // Ideally this should be information directly accessed from `functions` but it always returns an empty string. I don't know, I've been debuging this for a while now.
    func_strs: Vec<String>,

    integral_min_x: f64,
    integral_max_x: f64,

    integral_num: usize,

    help_open: bool,
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
                100,
                true,
                Some(def_min_x),
                Some(def_max_x),
                Some(def_interval),
            )],
            func_strs: vec![String::from(DEFAULT_FUNCION)],
            integral_min_x: def_min_x,
            integral_max_x: def_max_x,
            integral_num: def_interval,
            help_open: true,
        }
    }
}

impl epi::App for MathApp {
    // The name of the program (displayed when running natively as the window title)
    fn name(&self) -> &str { "Integral Demonstration" }

    // Called once before the first frame.
    fn setup(
        &mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>,
    ) {
    }

    // Called each time the UI needs repainting, which may be many times per second.
    #[inline(always)]
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        // Note: This Instant implementation does not show microseconds when using wasm.
        let start = instant::Instant::now();

        // Reduce size of final binary by just including one font
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Ubuntu-Light".to_owned(),
            FontData::from_static(include_bytes!("Ubuntu-Light.ttf")),
        );
        fonts.families.insert(
            FontFamily::Monospace,
            vec![
                "Ubuntu-Light".to_owned(),
            ],
        );
        fonts.families.insert(
            FontFamily::Proportional,
            vec![
                "Ubuntu-Light".to_owned(),
            ],
        );

        ctx.set_fonts(fonts);

        // Cute little window that lists supported functions!
        // TODO: add more detail
        egui::Window::new("Supported Functions")
            .default_pos([200.0, 200.0])
            .open(&mut self.help_open)
            .show(ctx, |ui| {
                ui.label("- sqrt, abs");
                ui.label("- exp, ln, log10 (log10 can also be called as log)");
                ui.label("- sin, cos, tan, asin, acos, atan, atan2");
                ui.label("- sinh, cosh, tanh, asinh, acosh, atanh");
                ui.label("- floor, ceil, round");
                ui.label("- signum, min, max");
            });

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Add Function")).clicked() {
                    // min_x and max_x will be updated later, doesn't matter here
                    self.functions.push(Function::new(
                        String::from(DEFAULT_FUNCION),
                        -1.0,
                        1.0,
                        100,
                        false,
                        None,
                        None,
                        None,
                    ));
                    self.func_strs.push(String::from(DEFAULT_FUNCION));
                }

                if ui.add(egui::Button::new("Open Help")).clicked() {
                    self.help_open = true;
                }
            });
        });

        let mut parse_error: String = "".to_string();
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                // ui.heading("Side Panel");

                let min_x_old = self.integral_min_x;
                let min_x_response =
                    ui.add(egui::Slider::new(&mut self.integral_min_x, X_RANGE.clone()).text("Min X"));

                let max_x_old = self.integral_max_x;
                let max_x_response = ui.add(egui::Slider::new(&mut self.integral_max_x, X_RANGE).text("Max X"));

                // Checks bounds, and if they are invalid, fix them
                if self.integral_min_x >= self.integral_max_x {
                    if max_x_response.changed() {
                        self.integral_max_x = max_x_old;
                    } else if min_x_response.changed() {
                        self.integral_min_x = min_x_old;
                    } else {
                        self.integral_min_x = -10.0;
                        self.integral_max_x = 10.0;
                    }
                }

                ui.add(egui::Slider::new(&mut self.integral_num, INTEGRAL_NUM_RANGE).text("Interval"));

                for (i, function) in self.functions.iter_mut().enumerate() {
                    let old_func_str = self.func_strs[i].clone();
                    let mut integral_toggle: bool = false;
                    ui.horizontal(|ui| {
                        ui.label("Function: ");
                        if ui.add(Button::new("Toggle Integral")).clicked() {
                            integral_toggle = true;
                        }
                        ui.text_edit_singleline(&mut self.func_strs[i]);
                    });

                    let integral: bool = if integral_toggle {
                        !function.is_integral()
                    } else {
                        function.is_integral()
                    };

                    let proc_func_str = add_asterisks(self.func_strs[i].clone());
                    let mut do_update: bool = true;
                    if !self.func_strs[i].is_empty() && (proc_func_str != old_func_str) {
                        let func_test_output = test_func(proc_func_str.clone());
                        if !func_test_output.is_empty() {
                            parse_error += &func_test_output;
                            do_update = false;
                        }
                    }
                    if do_update {
                        function.update(proc_func_str, integral, Some(self.integral_min_x), Some(self.integral_max_x), Some(self.integral_num));
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

        let step = (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64);

        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(format!("Error: {}", parse_error));
                return;
            }
            let available_width: usize = ui.available_width() as usize;

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

                    let mut i: usize = 0;
                    let mut functions_2: Vec<Function> = Vec::new();
                    for function_1 in self.functions.iter_mut() {
                        let function = function_1;
                        function.update_bounds(minx_bounds, maxx_bounds, available_width);

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
                        i += 1;
                        functions_2.push(function.clone());
                    }
                    self.functions = functions_2;
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
