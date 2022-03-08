use crate::function::{FunctionEntry, RiemannSum, EMPTY_FUNCTIONENTRY};
use crate::misc::digits_precision;
use crate::parsing::{add_asterisks, test_func};

use const_format::formatc;
use eframe::{egui, epi};
use egui::plot::Plot;
use egui::{
    Button, CentralPanel, Color32, ComboBox, Context, FontData, FontDefinitions, FontFamily,
    RichText, SidePanel, Slider, TopBottomPanel, Vec2, Visuals, Window,
};
use epi::{Frame, Storage};
use include_flate::flate;
use instant::Duration;
use shadow_rs::shadow;
use std::collections::BTreeMap;
use std::io::Read;
use std::ops::{BitXorAssign, RangeInclusive};

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

// Default values
pub const DEFAULT_FUNCION: &str = "x^2";
pub const DEFAULT_RIEMANN: RiemannSum = RiemannSum::Left;
const DEFAULT_MIN_X: f64 = -10.0;
const DEFAULT_MAX_X: f64 = 10.0;
const DEFAULT_INTEGRAL_NUM: usize = 100;

// Font Data
flate!(static FONTS_FILE: [u8] from "assets/fonts.tar");

lazy_static::lazy_static! {
    static ref FONT_DEFINITIONS: FontDefinitions = {
        let mut tar_archive = tar::Archive::new(&**FONTS_FILE);

        let mut ubuntu_light: Result<FontData, _> = Err("");
        let mut notoemoji: Result<FontData, _> = Err("");
        let mut hack: Result<FontData, _> = Err("");

        for file in tar_archive.entries().unwrap() {
            let mut file = file.unwrap();
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data).unwrap();
            let path = &file.header().path().unwrap();

            match (path).to_string_lossy().as_ref() {
                "Hack-Regular.ttf" => {
                    hack = Ok(FontData::from_owned(data))
                },
                "NotoEmoji-Regular.ttf" => {
                    notoemoji = Ok(FontData::from_owned(data))
                },
                "Ubuntu-Light.ttf" => {
                    ubuntu_light = Ok(FontData::from_owned(data))
                },
                _ => {
                    panic!("Other files in this archive!!");
                }
            }
        }

        let mut font_data: BTreeMap<String, FontData> = BTreeMap::new();
        let mut families = BTreeMap::new();

        font_data.insert(
            "Hack".to_owned(),
            hack.unwrap(),
        );
        font_data.insert("Ubuntu-Light".to_owned(), ubuntu_light.unwrap());
        font_data.insert("NotoEmoji-Regular".to_owned(), notoemoji.unwrap());

        families.insert(
            FontFamily::Monospace,
            vec!["Hack".to_owned(), "Ubuntu-Light".to_owned(), "NotoEmoji-Regular".to_owned()],
        );

        families.insert(
            FontFamily::Proportional,
            vec!["Ubuntu-Light".to_owned(), "NotoEmoji-Regular".to_owned()],
        );

        FontDefinitions {
            font_data,
            families,
        }
    };
}

// Used when displaying supported expressions in the Help window
const HELP_EXPR: &str = "- sqrt(x): square root of x
- abs(x): absolute value of x
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
- floor, ceil, round, signum";

const HELP_VARS: &str = "- Euler's number is supported via 'E' (noted it being uppercase)
- PI is available through 'pi' or 'π'";

// Used in the "Panel" section of the Help window
const HELP_PANEL: &str =
"- The 'Panel' button on the top bar toggles if the side bar should be shown or not.
- The 'Add Function' button on the top panel adds a new function to be graphed. You can then configure that function in the side panel.
- The 'Help' button on the top bar opens and closes this window!
- The 'Info' button provides information on the build currently running.
- The Sun/Moon button toggles Dark and Light mode.";

// Used in the "Functions" section of the Help window
const HELP_FUNCTION: &str = "- The 'X' button before the '∫' button allows you to delete the function in question. Deleting a function is prevented if only 1 function exists.
- The ∫ button (between the 'X' and 'd/dx' buttons) indicates whether to integrate the function in question.
- The 'd/dx' button next to the function input indicates whether or not calculating the derivative is enabled or not.";

// Misc help info
const HELP_OTHER: &str =
"- In some edge cases, math functions may not parse correctly. More specifically with implicit multiplication. If you incounter this issue, please do report it on the project's Github page (linked on the side panel). But a current workaround would be explicitly stating a multiplication operation through the use of an asterisk.";

// Used to provide info on the Licensing of the project
const LICENSE_INFO: &str = "The AGPL license ensures that the end user, even if not hosting the program itself, is still guaranteed access to the source code of the project in question.";

// The URL of the project
const PROJECT_URL: &str = "https://github.com/Titaniumtown/YTBN_graphing_software";

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

    // Stores whether or not dark mode is enabled
    pub dark_mode: bool,
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
            dark_mode: true,
        }
    }
}

pub struct MathApp {
    // Stores vector of functions
    functions: Vec<FunctionEntry>,

    // Stores vector containing the string representation of the functions. This is used because of hacky reasons
    func_strs: Vec<String>,

    // Stores last error from parsing functions (used to display the same error when side panel is minimized)
    last_error: Vec<(usize, String)>,

    // Contains the list of Areas calculated (the vector of f64) and time it took for the last frame (the Duration). Stored in a Tuple.
    last_info: (Vec<f64>, Duration),

    // Stores Settings (pretty self explanatory)
    settings: AppSettings,
}

impl Default for MathApp {
    fn default() -> Self {
        Self {
            functions: vec![EMPTY_FUNCTIONENTRY.clone().integral(true)],
            func_strs: vec![String::from(DEFAULT_FUNCION)],
            last_error: Vec::new(),
            last_info: (vec![0.0], Duration::ZERO),
            settings: AppSettings::default(),
        }
    }
}

impl MathApp {
    fn side_panel(&mut self, ctx: &Context) {
        // Side Panel which contains vital options to the operation of the application (such as adding functions and other options)
        SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ComboBox::from_label("Riemann Sum Type")
                    .selected_text(self.settings.sum.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.settings.sum, RiemannSum::Left, "Left");
                        ui.selectable_value(&mut self.settings.sum, RiemannSum::Middle, "Middle");
                        ui.selectable_value(&mut self.settings.sum, RiemannSum::Right, "Right");
                    });

                let min_x_old = self.settings.integral_min_x;
                let min_x_changed = ui
                    .add(
                        Slider::new(&mut self.settings.integral_min_x, INTEGRAL_X_RANGE)
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

                let functions_len = self.functions.len();
                let mut remove_i: Option<usize> = None;
                for (i, function) in self.functions.iter_mut().enumerate() {
                    let integral_enabled = function.integral;
                    let derivative_enabled = function.derivative;
                    let mut derivative_toggle: bool = false;
                    let mut integral_toggle: bool = false;

                    // Entry for a function
                    ui.horizontal(|ui| {
                        ui.label("Function:");

                        if functions_len > 1 {
                            // There's more than 1 function! Functions can now be deleted
                            if ui
                                .add(Button::new("X"))
                                .on_hover_text("Delete Function")
                                .clicked()
                            {
                                remove_i = Some(i);
                            }
                        } else {
                            // Display greyed out "X" button if there's only one function added
                            ui.add_enabled(false, Button::new("X"));
                        }

                        integral_toggle = ui
                            .add(Button::new("∫"))
                            .on_hover_text(match integral_enabled {
                                true => "Don't integrate",
                                false => "Integrate",
                            })
                            .clicked();

                        derivative_toggle = ui
                            .add(Button::new("d/dx"))
                            .on_hover_text(match derivative_enabled {
                                true => "Don't Differentiate",
                                false => "Differentiate",
                            })
                            .clicked();

                        ui.text_edit_singleline(&mut self.func_strs[i]);
                    });

                    let proc_func_str = add_asterisks(self.func_strs[i].clone());
                    if integral_toggle
                        | derivative_toggle
                        | max_x_changed
                        | min_x_changed
                        | (proc_func_str != function.get_func_str())
                        | self.last_error.iter().any(|ele| ele.0 == i)
                    {
                        // let proc_func_str = self.func_strs[i].clone();
                        let func_test_output = test_func(&proc_func_str);
                        if let Some(test_output_value) = func_test_output {
                            self.last_error.push((i, test_output_value));
                        } else {
                            function.update(
                                proc_func_str,
                                if integral_toggle {
                                    !integral_enabled
                                } else {
                                    integral_enabled
                                },
                                if derivative_toggle {
                                    !derivative_enabled
                                } else {
                                    derivative_enabled
                                },
                                Some(self.settings.integral_min_x),
                                Some(self.settings.integral_max_x),
                                Some(self.settings.integral_num),
                                Some(self.settings.sum),
                            );
                            self.last_error = self
                                .last_error
                                .iter()
                                .filter(|(i_ele, _)| i_ele != &i)
                                .map(|(a, b)| (*a, b.clone()))
                                .collect();
                        }
                    }
                }

                if self.functions.len() > 1 {
                    if let Some(remove_i_unwrap) = remove_i {
                        self.functions.remove(remove_i_unwrap);
                        self.func_strs.remove(remove_i_unwrap);
                    }
                }

                // Open Source and Licensing information
                ui.hyperlink_to("I'm Opensource!", PROJECT_URL);

                ui.label(RichText::new("(and licensed under AGPLv3)").color(Color32::LIGHT_GRAY))
                    .on_hover_text(LICENSE_INFO);
            });
    }
}

impl epi::App for MathApp {
    // The name of the program (displayed when running natively as the window title)
    fn name(&self) -> &str { "(Yet-to-be-named) Graphing Software" }

    // Called once before the first frame.
    fn setup(&mut self, _ctx: &Context, _frame: &Frame, _storage: Option<&dyn Storage>) {}

    // Called each time the UI needs repainting, which may be many times per second.
    #[inline(always)]
    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        let start = instant::Instant::now();
        ctx.set_visuals(match self.settings.dark_mode {
            true => Visuals::dark(),
            false => Visuals::light(),
        });

        ctx.set_fonts(FONT_DEFINITIONS.clone()); // Initialize fonts

        // Creates Top bar that contains some general options
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.settings.show_side_panel.bitxor_assign(
                    ui.add(Button::new("Panel"))
                        .on_hover_text(match self.settings.show_side_panel {
                            true => "Hide Side Panel",
                            false => "Show Side Panel",
                        })
                        .clicked(),
                );

                if ui
                    .add(Button::new("Add Function"))
                    .on_hover_text("Create and graph new function")
                    .clicked()
                {
                    self.functions.push(
                        EMPTY_FUNCTIONENTRY
                            .clone()
                            .update_riemann(self.settings.sum),
                    );
                    self.func_strs.push(String::new());
                }

                self.settings.help_open.bitxor_assign(
                    ui.add(Button::new("Help"))
                        .on_hover_text(match self.settings.help_open {
                            true => "Close Help Window",
                            false => "Open Help Window",
                        })
                        .clicked(),
                );

                self.settings.info_open.bitxor_assign(
                    ui.add(Button::new("Info"))
                        .on_hover_text(match self.settings.info_open {
                            true => "Close Info Window",
                            false => "Open Info Window",
                        })
                        .clicked(),
                );

                self.settings.dark_mode.bitxor_assign(
                    ui.add(Button::new(match self.settings.dark_mode {
                        true => "🌞",
                        false => "🌙",
                    }))
                    .on_hover_text(match self.settings.dark_mode {
                        true => "Turn the Lights on!",
                        false => "Turn the Lights off.",
                    })
                    .clicked(),
                );

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
                ui.heading("Help With...");

                ui.collapsing("Supported Expressions", |ui| {
                    ui.label(HELP_EXPR);
                });

                ui.collapsing("Supported Constants", |ui| {
                    ui.label(HELP_VARS);
                });

                ui.collapsing("Panel", |ui| {
                    ui.label(HELP_PANEL);
                });

                ui.collapsing("Functions", |ui| {
                    ui.label(HELP_FUNCTION);
                });

                ui.collapsing("Other", |ui| {
                    ui.label(HELP_OTHER);
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

        // Central panel which contains the central plot (or an error created when parsing)
        CentralPanel::default().show(ctx, |ui| {
            if !self.last_error.is_empty() {
                ui.centered_and_justified(|ui| {
                    self.last_error.iter().for_each(|ele| {
                        ui.heading(&(&format!("(Function #{}) {}\n", ele.0, ele.1)).to_string());
                    })
                });
                return;
            }

            let available_width: usize = ui.available_width() as usize;
            let step = (self.settings.integral_min_x - self.settings.integral_max_x).abs()
                / (self.settings.integral_num as f64);

            Plot::new("plot")
                .set_margin_fraction(Vec2::ZERO)
                .data_aspect(1.0)
                .include_y(0)
                .show(ui, |plot_ui| {
                    let bounds = plot_ui.plot_bounds();
                    let minx_bounds: f64 = bounds.min()[0];
                    let maxx_bounds: f64 = bounds.max()[0];

                    let area_list: Vec<f64> = self
                        .functions
                        .iter_mut()
                        .enumerate()
                        .map(|(i, function)| {
                            if self.func_strs[i].is_empty() {
                                return f64::NAN;
                            }

                            function.update_bounds(minx_bounds, maxx_bounds, available_width);

                            let (back_values, integral, derivative) = function.run();
                            let func_str = function.get_func_str();
                            plot_ui.line(back_values.color(Color32::RED).name(func_str));

                            if let Some(derivative_data) = derivative {
                                plot_ui.line(
                                    derivative_data
                                        .color(Color32::GREEN)
                                        .name(function.get_derivative_str()),
                                );
                            }

                            if let Some(integral_data) = integral {
                                let integral_name = format!("Integral of {}", func_str);
                                plot_ui.bar_chart(
                                    integral_data
                                        .0
                                        .color(Color32::BLUE)
                                        .width(step)
                                        .name(integral_name),
                                );

                                digits_precision(integral_data.1, 8)
                            } else {
                                f64::NAN
                            }
                        })
                        .collect();
                    self.last_info = (area_list, start.elapsed());
                });
        });
    }

    // Uncaps max canvas size. This was capped in egui due to a bug in Firefox. But it's fixed now.
    fn max_size_points(&self) -> Vec2 { Vec2::new(f32::MAX, f32::MAX) }
}
