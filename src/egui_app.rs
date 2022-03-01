use std::ops::RangeInclusive;

use crate::function::{Function, RiemannSum};
use crate::misc::{add_asterisks, digits_precision, test_func};
use eframe::{egui, epi};
use egui::plot::Plot;
use egui::widgets::Button;
use egui::{Color32, FontData, FontFamily, Vec2};
use git_version::git_version;
use include_flate::flate;

// Grabs git version on compile time
const GIT_VERSION: &str = git_version!();

// Sets some hard-coded limits to the application
const INTEGRAL_NUM_RANGE: RangeInclusive<usize> = 1..=100000;
const INTEGRAL_X_MIN: f64 = -1000.0;
const INTEGRAL_X_MAX: f64 = 1000.0;
const INTEGRAL_X_RANGE: RangeInclusive<f64> = INTEGRAL_X_MIN..=INTEGRAL_X_MAX;

const DEFAULT_FUNCION: &str = "x^2"; // Default function that appears when adding a new function
const DEFAULT_RIEMANN: RiemannSum = RiemannSum::Left;
const DEFAULT_MIN_X: f64 = -10.0;
const DEFAULT_MAX_X: f64 = 10.0;
const DEFAULT_INTEGRAL_NUM: usize = 100;

flate!(static FONT_DATA: [u8] from "assets/Ubuntu-Light.ttf"); // Font used when displaying text

// Used when displaying supported expressions in the Help window
const HELP1_TEXT: &str = "- sqrt(x): square root of x
- abs(x): absolute value of x
- exp(x): e^x
- ln(x): log with base e
- log10(x): base 10 logarithm of x
- log(x): same as log10(x)
- sin(x): Sine of x
- cos(x): Cosine of x
- tan(x): Tangent of x
- asin(x): arcsine of x
- acos(x): arccosine of x
- atan(x): arctangent of x
- atan2, sinh, cosh, tanh, asinh, acosh, atanh
- floor, ceil, round
- signum, min, max";

// Used in the "Buttons" section of the Help window
const HELP2_TEXT: &str = "- The ∫ button next to the function input indicates whether estimating an integral for that function is enabled or not.
- The 'Add Function' button on the top panel adds a new function to be graphed. You can then configure that function in the side panel.
- The 'Open Help' Button on the top bar opens and closes this window!
- The 'Open Info' Button on the top bar opens and closes a window which contains info such as the estimated area for each function, and the time it took to render the last frame.";

pub struct MathApp {
    // Stores vector of functions
    functions: Vec<Function>,

    // Stores vector containing the string representation of the functions. This is used because of hacky reasons
    func_strs: Vec<String>,

    // Min and Max range for calculating an integral
    integral_min_x: f64,
    integral_max_x: f64,

    // Number of rectangles used to calculate integral
    integral_num: usize,

    // Stores whether or not the help window is open
    help_open: bool,

    // Stores whether or not the info window is open
    info_open: bool,

    // Stores font data that's used when displaying text
    font: FontData,

    // Stores the type of Rienmann sum that should be calculated
    sum: RiemannSum,
}

impl Default for MathApp {
    fn default() -> Self {
        Self {
            functions: vec![Function::new(
                String::from(DEFAULT_FUNCION),
                DEFAULT_MIN_X,
                DEFAULT_MAX_X,
                100,  // Doesn't matter as it will be overwritten
                true, // Enables integral
                Some(DEFAULT_MIN_X),
                Some(DEFAULT_MAX_X),
                Some(DEFAULT_INTEGRAL_NUM),
                Some(DEFAULT_RIEMANN),
            )],
            func_strs: vec![String::from(DEFAULT_FUNCION)],
            integral_min_x: DEFAULT_MIN_X,
            integral_max_x: DEFAULT_MAX_X,
            integral_num: DEFAULT_INTEGRAL_NUM,
            help_open: true,
            info_open: false,
            font: FontData::from_static(&FONT_DATA),
            sum: DEFAULT_RIEMANN,
        }
    }
}

impl MathApp {
    // Sets up fonts to use Ubuntu-Light
    fn init_font(&self, ctx: &egui::Context) {
        // Reduce size of final binary by just including one font
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("Ubuntu-Light".to_owned(), self.font.clone());
        fonts
            .families
            .insert(FontFamily::Monospace, vec!["Ubuntu-Light".to_owned()]);
        fonts
            .families
            .insert(FontFamily::Proportional, vec!["Ubuntu-Light".to_owned()]);

        ctx.set_fonts(fonts);
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

        self.init_font(ctx); // Setup fonts

        // Creates Top bar that contains some general options
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Add Function")).clicked() {
                    self.functions.push(Function::new(
                        String::from(DEFAULT_FUNCION),
                        -1.0, // Doesn't matter, updated later
                        1.0,  // Doesn't matter, updated later
                        100,  // Doesn't matter, updated later
                        false,
                        None, // Doesn't matter, updated later
                        None, // Doesn't matter, updated later
                        None, // Doesn't matter, updated later
                        Some(self.sum),
                    ));
                    self.func_strs.push(String::from(DEFAULT_FUNCION));
                }

                if ui.add(egui::Button::new("Open Help")).clicked() {
                    self.help_open = !self.help_open;
                }

                if ui.add(egui::Button::new("Open Info")).clicked() {
                    self.info_open = !self.info_open;
                }
            });
        });

        // Cute little window that lists supported functions!
        egui::Window::new("Help")
            .default_pos([200.0, 200.0])
            .open(&mut self.help_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Supported Expressions");
                ui.label(HELP1_TEXT);

                ui.heading("Buttons");
                ui.label(HELP2_TEXT);
            });

        let mut parse_error: String = String::new(); // Stores errors found when interpreting input functions

        // Side Panel which contains vital options to the operation of the application (such as adding functions and other options)
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Riemann Sum Type")
                    .selected_text(self.sum.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sum, RiemannSum::Left, "Left");
                        ui.selectable_value(&mut self.sum, RiemannSum::Middle, "Middle");
                        ui.selectable_value(&mut self.sum, RiemannSum::Right, "Right");
                    });

                let min_x_old = self.integral_min_x;
                let min_x_response =
                    ui.add(egui::Slider::new(&mut self.integral_min_x, INTEGRAL_X_RANGE.clone()).text("Min X"));

                let max_x_old = self.integral_max_x;
                let max_x_response = ui.add(egui::Slider::new(&mut self.integral_max_x, INTEGRAL_X_RANGE).text("Max X"));

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
                    let mut integral_toggle: bool = false;

                    // Entry for a function
                    ui.horizontal(|ui| {
                        ui.label("Function: ");
                        if ui.add(Button::new("∫")).clicked() {
                            integral_toggle = true;
                        }
                        ui.text_edit_singleline(&mut self.func_strs[i]);
                    });

                    let integral: bool = if integral_toggle {
                        !function.integral
                    } else {
                        function.integral
                    };

                    if !self.func_strs[i].is_empty() {
                        let proc_func_str = add_asterisks(self.func_strs[i].clone());
                        let func_test_output = test_func(proc_func_str.clone());
                        if let Some(test_output_value) = func_test_output {
                            parse_error += &format!("(Function #{}) {}", i, test_output_value);
                        } else {
                            function.update(proc_func_str, integral, Some(self.integral_min_x), Some(self.integral_max_x), Some(self.integral_num), Some(self.sum));
                        }
                    } else {
                        function.func_str = "".to_string();
                    }
                }

                // Open Source and Licensing information
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
                    if GIT_VERSION.contains("-modified") {
                        // If git version is modified, don't display a link to the commit on github (as the commit will not exist)
                        ui.label(GIT_VERSION);
                    } else {
                        ui.hyperlink_to(
                            GIT_VERSION,
                            format!(
                                "https://github.com/Titaniumtown/integral_site/commit/{}",
                                GIT_VERSION
                            ),
                        );
                    }
                });
            });

        let step = (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64);

        let mut area_list: Vec<f64> = Vec::new(); // Stores list of areas resulting from calculating the integral of functions

        // Stores the final Plot
        egui::CentralPanel::default().show(ctx, |ui| {
            if !parse_error.is_empty() {
                ui.label(parse_error);
                return;
            }
            let available_width: usize = ui.available_width() as usize;

            Plot::new("plot")
                .set_margin_fraction(Vec2::ZERO)
                // .view_aspect(1.0)
                .data_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    let bounds = plot_ui.plot_bounds();
                    let minx_bounds: f64 = bounds.min()[0];
                    let maxx_bounds: f64 = bounds.max()[0];

                    let mut i: usize = 0;
                    for function in self.functions.iter_mut() {
                        if self.func_strs[i].is_empty() {
                            continue;
                        }

                        function.update_bounds(minx_bounds, maxx_bounds, available_width);

                        let (back_values, bars) = function.run();
                        plot_ui.line(back_values.color(Color32::RED));

                        if let Some(bars_data) = bars {
                            let (bar_chart, area) = bars_data;
                            plot_ui.bar_chart(bar_chart.color(Color32::BLUE).width(step));
                            area_list.push(digits_precision(area, 8))
                        } else {
                            area_list.push(f64::NAN);
                        }
                        i += 1;
                    }
                });
        });

        let duration = start.elapsed();

        egui::Window::new("Info")
            .default_pos([200.0, 200.0])
            .resizable(false)
            .collapsible(false)
            .open(&mut self.info_open)
            .show(ctx, |ui| {
                // Displays all areas of functions along with how long it took to complete the entire frame
                ui.label(format!(
                    "Area: {:?} Took: {:?}",
                    area_list.clone(),
                    duration
                ));
            });
    }
}
