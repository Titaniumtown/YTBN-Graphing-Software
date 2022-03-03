use crate::function::{Function, RiemannSum};
use crate::misc::{add_asterisks, digits_precision, test_func};
use const_format::formatc;
use eframe::{egui, epi};
use egui::plot::Plot;
use egui::{
    Button, CentralPanel, Color32, ComboBox, Context, FontData, FontDefinitions, FontFamily,
    RichText, SidePanel, Slider, TopBottomPanel, Vec2, Window,
};
use epi::{Frame, Storage};
use include_flate::flate;
use instant::Duration;
use shadow_rs::shadow;
use std::ops::RangeInclusive;

shadow!(build);

// Constant string that has a string containing information about the build.
const BUILD_INFO: &str = formatc!(
    "Commit: {} ({})\nBuild Date: {}\nRust Channel: {}\nRust Version: {}",
    &build::SHORT_COMMIT,
    &build::BRANCH,
    &build::BUILD_TIME,
    &build::RUST_CHANNEL,
    &build::RUST_VERSION,
);

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
const HELP_EXPR: &str = "- sqrt(x): square root of x
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
const HELP_BUTTONS: &str = "- The 'Panel' button on the top bar toggles if the side bar should be shown or not.
- The ∫ button next to the function input indicates whether estimating an integral for that function is enabled or not.
- The 'Add Function' button on the top panel adds a new function to be graphed. You can then configure that function in the side panel.
- The 'Help' button on the top bar opens and closes this window!
- The 'Info' button provides information on the build currently running.";

const HELP_MISC: &str = "- In some edge cases, math functions may not parse correctly. More specifically with implicit multiplication. If you incounter this issue, please do report it on the project's Github page (linked on the side panel). But a bypass would be explicitly stating a multiplication operation through the use of an asterisk
- A (very minor) note in regards to the timing functionality (the 'took' number in the top panel): this value is not accurate when running in the browser in comparison to when running natively. Implementations of this timing functionality vary from browser-to-browser.";

// Used to provide info on the Licensing of the project
const LICENSE_INFO: &str = "The AGPL license ensures that the end user, even if not hosting the program itself, still is guaranteed access to the source code of the project in question.";

// Stores settings
struct AppSettings {
    // Stores whether or not the Help window is open
    pub help_open: bool,

    // Stores whether or not the Info window is open
    pub info_open: bool,

    // Stores whether or not the side panel is shown or not
    pub show_side_panel: bool,

    // Stores the type of Rienmann sum that should be calculated
    pub sum: RiemannSum,

    // Min and Max range for calculating an integral
    pub integral_min_x: f64,
    pub integral_max_x: f64,

    // Number of rectangles used to calculate integral
    pub integral_num: usize,

    // Stores whether or not the integral functionality is being used
    pub integral_used: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            help_open: true,
            info_open: false,
            show_side_panel: true,
            sum: DEFAULT_RIEMANN,
            integral_min_x: DEFAULT_MIN_X,
            integral_max_x: DEFAULT_MAX_X,
            integral_num: DEFAULT_INTEGRAL_NUM,
            integral_used: true,
        }
    }
}

pub struct MathApp {
    // Stores vector of functions
    functions: Vec<Function>,

    // Stores vector containing the string representation of the functions. This is used because of hacky reasons
    func_strs: Vec<String>,

    // Stores last error from parsing functions (used to display the same error when side panel is minimized)
    last_error: String,

    // Stores font data that's used when displaying text
    font: FontData,

    // Contains the list of Areas calculated (the vector of f64) and time it took for the last frame (the Duration). Stored in a Tuple.
    last_info: (Vec<f64>, Duration),

    // Stores Settings (pretty self explanatory)
    settings: AppSettings,
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
            last_error: String::new(),
            font: FontData::from_static(&FONT_DATA),
            last_info: (vec![0.0], Duration::ZERO),
            settings: AppSettings::default(),
        }
    }
}

impl MathApp {
    // Sets up fonts to use Ubuntu-Light
    fn init_font(&self, ctx: &Context) {
        // Reduce size of final binary by just including one font
        let mut fonts = FontDefinitions::default();
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

    fn side_panel(&mut self, ctx: &Context) {
        // Side Panel which contains vital options to the operation of the application (such as adding functions and other options)
        SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                if self.settings.integral_used {
                    ComboBox::from_label("Riemann Sum Type")
                        .selected_text(self.settings.sum.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.settings.sum, RiemannSum::Left, "Left");
                            ui.selectable_value(
                                &mut self.settings.sum,
                                RiemannSum::Middle,
                                "Middle",
                            );
                            ui.selectable_value(&mut self.settings.sum, RiemannSum::Right, "Right");
                        });
                }

                let min_x_old = self.settings.integral_min_x;
                let min_x_changed = ui
                    .add(
                        Slider::new(&mut self.settings.integral_min_x, INTEGRAL_X_RANGE.clone())
                            .text("Min X"),
                    )
                    .changed();

                let max_x_old = self.settings.integral_max_x;
                let max_x_changed = ui
                    .add(
                        Slider::new(&mut self.settings.integral_max_x, INTEGRAL_X_RANGE)
                            .text("Max X"),
                    )
                    .changed();

                // Checks bounds, and if they are invalid, fix them
                if self.settings.integral_min_x >= self.settings.integral_max_x {
                    if max_x_changed {
                        self.settings.integral_max_x = max_x_old;
                    } else if min_x_changed {
                        self.settings.integral_min_x = min_x_old;
                    } else {
                        // No clue how this would happen, but just in case
                        self.settings.integral_min_x = DEFAULT_MIN_X;
                        self.settings.integral_max_x = DEFAULT_MAX_X;
                    }
                }

                ui.add(
                    Slider::new(&mut self.settings.integral_num, INTEGRAL_NUM_RANGE)
                        .text("Interval"),
                );

                let mut remove_i: Option<usize> = None;
                let mut using_integral: bool = false;
                self.last_error = String::new();
                for (i, function) in self.functions.iter_mut().enumerate() {
                    let mut integral_toggle: bool = false;
                    let integral_enabled = function.integral;
                    // Entry for a function
                    ui.horizontal(|ui| {
                        ui.label("Function:");
                        if ui
                            .add(Button::new("X"))
                            .on_hover_text("Delete Function")
                            .clicked()
                        {
                            remove_i = Some(i);
                        }
                        if ui
                            .add(Button::new("∫"))
                            .on_hover_text(if integral_enabled {
                                "Don't integrate"
                            } else {
                                "Integrate"
                            })
                            .clicked()
                        {
                            integral_toggle = true;
                        }
                        ui.text_edit_singleline(&mut self.func_strs[i]);
                    });

                    let integral: bool = if integral_toggle {
                        !integral_enabled
                    } else {
                        integral_enabled
                    };

                    if integral {
                        using_integral = true;
                    }

                    if !self.func_strs[i].is_empty() {
                        let proc_func_str = add_asterisks(self.func_strs[i].clone());
                        let func_test_output = test_func(proc_func_str.clone());
                        if let Some(test_output_value) = func_test_output {
                            self.last_error += &format!("(Function #{}) {}", i, test_output_value);
                        } else {
                            function.update(
                                proc_func_str,
                                integral,
                                Some(self.settings.integral_min_x),
                                Some(self.settings.integral_max_x),
                                Some(self.settings.integral_num),
                                Some(self.settings.sum),
                            );
                        }
                    } else {
                        function.func_str = String::new();
                    }
                }

                self.settings.integral_used = using_integral;

                if self.functions.len() > 1 {
                    if let Some(remove_i_unwrap) = remove_i {
                        self.functions.remove(remove_i_unwrap);
                        self.func_strs.remove(remove_i_unwrap);
                    }
                }

                // Open Source and Licensing information
                ui.hyperlink_to(
                    "I'm Opensource!",
                    "https://github.com/Titaniumtown/integral_site",
                );
                ui.label(RichText::new("(and licensed under AGPLv3)").color(Color32::LIGHT_GRAY))
                    .on_hover_text(LICENSE_INFO);
            });
    }
}

impl epi::App for MathApp {
    // The name of the program (displayed when running natively as the window title)
    fn name(&self) -> &str { "Integral Demonstration" }

    // Called once before the first frame.
    fn setup(&mut self, _ctx: &Context, _frame: &Frame, _storage: Option<&dyn Storage>) {}

    // Called each time the UI needs repainting, which may be many times per second.
    #[inline(always)]
    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        // Note: This Instant implementation does not show microseconds when using wasm.
        let start = instant::Instant::now();

        self.init_font(ctx); // Setup fonts

        // Creates Top bar that contains some general options
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(Button::new("Panel"))
                    .on_hover_text(match self.settings.show_side_panel {
                        true => "Hide Side Panel",
                        false => "Show Side Panel",
                    })
                    .clicked()
                {
                    self.settings.show_side_panel = !self.settings.show_side_panel;
                }

                if ui
                    .add(Button::new("Add Function"))
                    .on_hover_text("Create and graph new function")
                    .clicked()
                {
                    self.functions.push({
                        let mut function = Function::new(
                            String::from(DEFAULT_FUNCION),
                            -1.0, // Doesn't matter, updated later
                            1.0,  // Doesn't matter, updated later
                            100,  // Doesn't matter, updated later
                            false,
                            None, // Doesn't matter, updated later
                            None, // Doesn't matter, updated later
                            None, // Doesn't matter, updated later
                            Some(self.settings.sum),
                        );
                        function.func_str = String::new();
                        function
                    });
                    self.func_strs.push(String::new());
                }

                if ui
                    .add(Button::new("Help"))
                    .on_hover_text("Open Help Window")
                    .clicked()
                {
                    self.settings.help_open = !self.settings.help_open;
                }

                if ui
                    .add(Button::new("Info"))
                    .on_hover_text("Show Info")
                    .clicked()
                {
                    self.settings.info_open = !self.settings.info_open;
                }

                ui.label(format!(
                    "Area: {:?} Took: {:?}",
                    self.last_info.0, self.last_info.1
                ));
            });
        });

        // Help window with information for users
        Window::new("Help")
            .default_pos([200.0, 200.0])
            .open(&mut self.settings.help_open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.collapsing("Supported Expressions", |ui| {
                    ui.label(HELP_EXPR);
                });

                ui.collapsing("Buttons", |ui| {
                    ui.label(HELP_BUTTONS);
                });

                ui.collapsing("Misc", |ui| {
                    ui.label(HELP_MISC);
                });
            });

        // Window with Misc Information
        Window::new("Info")
            .default_pos([200.0, 200.0])
            .open(&mut self.settings.info_open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(&*BUILD_INFO);
            });

        // If side panel is enabled, show it.
        if self.settings.show_side_panel {
            self.side_panel(ctx);
        }

        // Central panel which contains the central plot along or an error when parsing
        CentralPanel::default().show(ctx, |ui| {
            if !self.last_error.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading(self.last_error.clone());
                });
                return;
            }

            let available_width: usize = ui.available_width() as usize;

            Plot::new("plot")
                .set_margin_fraction(Vec2::ZERO)
                .data_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    let bounds = plot_ui.plot_bounds();
                    let minx_bounds: f64 = bounds.min()[0];
                    let maxx_bounds: f64 = bounds.max()[0];

                    let step = (self.settings.integral_min_x - self.settings.integral_max_x).abs()
                        / (self.settings.integral_num as f64);

                    let mut area_list: Vec<f64> = Vec::new(); // Stores list of areas resulting from calculating the integral of functions

                    for (i, function) in self.functions.iter_mut().enumerate() {
                        if self.func_strs[i].is_empty() {
                            continue;
                        }

                        function.update_bounds(minx_bounds, maxx_bounds, available_width);

                        let (back_values, bars) = function.run();
                        plot_ui.line(back_values.color(Color32::RED));

                        area_list.push({
                            if let Some(bars_data) = bars {
                                let (bar_chart, area) = bars_data;
                                plot_ui.bar_chart(bar_chart.color(Color32::BLUE).width(step));
                                digits_precision(area, 8)
                            } else {
                                f64::NAN
                            }
                        });
                    }
                    self.last_info = (area_list, start.elapsed());
                });
        });
    }

    // Uncaps max canvas size. This was capped in egui due to a bug in Firefox. But it's fixed now.
    fn max_size_points(&self) -> Vec2 { Vec2::new(f32::MAX, f32::MAX) }
}
