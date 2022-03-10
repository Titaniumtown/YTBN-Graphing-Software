use crate::function::{FunctionEntry, RiemannSum, EMPTY_FUNCTION_ENTRY};
use crate::misc::{debug_log, log_helper};
use crate::parsing::{process_func_str, test_func};

use const_format::formatc;
use eframe::{egui, epi};
use egui::plot::Plot;
use egui::{
	Button, CentralPanel, Color32, ComboBox, Context, FontData, FontDefinitions, FontFamily,
	RichText, SidePanel, Slider, TopBottomPanel, Vec2, Visuals, Window,
};
use epi::{Frame, Storage};
use instant::Duration;
use shadow_rs::shadow;
use std::{
	collections::BTreeMap,
	io::Read,
	ops::{BitXorAssign, RangeInclusive},
	str,
};

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

// Stores data loaded from files
struct FileData {
	// Stores fonts
	pub font_ubuntu_light: FontData,
	pub font_notoemoji: FontData,
	pub font_hack: FontData,

	// Stores text
	pub text_help_expr: String,
	pub text_help_vars: String,
	pub text_help_panel: String,
	pub text_help_function: String,
	pub text_help_other: String,
}

lazy_static::lazy_static! {
	// Load all of the data from the compressed tarball
	static ref FILE_DATA: FileData = {
		let start = instant::Instant::now();
		log_helper("Loading assets...");
		let mut tar_file_raw = include_bytes!("../data.tar.zst").as_slice();
		log_helper("Decompressing...");
		let mut tar_file = ruzstd::StreamingDecoder::new(&mut tar_file_raw).unwrap();
		let mut tar_file_data = Vec::new();
		tar_file.read_to_end(&mut tar_file_data).unwrap();

		let mut tar_archive = tar::Archive::new(&*tar_file_data);

		// Stores fonts
		let mut font_ubuntu_light: Option<FontData> = None;
		let mut font_notoemoji: Option<FontData> = None;
		let mut font_hack: Option<FontData> = None;

		// Stores text
		let mut text_help_expr: Option<String> = None;
		let mut text_help_vars: Option<String> = None;
		let mut text_help_panel: Option<String> = None;
		let mut text_help_function: Option<String> = None;
		let mut text_help_other: Option<String> = None;


		log_helper("Reading asset files...");
		// Iterate through all entries in the tarball
		for file in tar_archive.entries().unwrap() {
			let mut file = file.unwrap();
			let mut data: Vec<u8> = Vec::new();
			file.read_to_end(&mut data).unwrap();
			let path = file.header().path().unwrap();
			let path_string = path.to_string_lossy();

			debug_log(&format!("Loading file: {}", path_string));

			// Match the filename
			if path_string.ends_with(".ttf") {
				// Parse font files
				let font_data = FontData::from_owned(data);
				match path_string.as_ref() {
					"Hack-Regular.ttf" => {
						font_hack = Some(font_data);
					},
					"NotoEmoji-Regular.ttf" => {
						font_notoemoji = Some(font_data);
					},
					"Ubuntu-Light.ttf" => {
						font_ubuntu_light = Some(font_data);
					},
					_ => {
						panic!("Font File {} not expected!", path_string);
					}
				}
			} else if path_string.ends_with(".txt") {
				// Parse text files
				let string_data = str::from_utf8(&data).unwrap().to_string();
				match path_string.as_ref() {
					"text_help_expr.txt" => {
						text_help_expr = Some(string_data);
					},
					"text_help_vars.txt" => {
						text_help_vars = Some(string_data);
					},
					"text_help_panel.txt" => {
						text_help_panel = Some(string_data);
					},
					"text_help_function.txt" => {
						text_help_function = Some(string_data);
					},
					"text_help_other.txt" => {
						text_help_other = Some(string_data);
					},
					_ => {
						panic!("Text file {} not expected!", path_string);
					}
				}
			} else {
				panic!("Other file {} not expected!", path_string);
			}
		}

		log_helper(&format!("Done loading assets! Took: {:?}", start.elapsed()));

		// Create and return FileData struct
		FileData {
			font_ubuntu_light: font_ubuntu_light.expect("Ubuntu Light font not found!"),
			font_notoemoji: font_notoemoji.expect("Noto Emoji font not found!"),
			font_hack: font_hack.expect("Hack font not found!"),
			text_help_expr: text_help_expr.expect("HELP_EXPR not found"),
			text_help_vars: text_help_vars.expect("HELP_VARS not found"),
			text_help_panel: text_help_panel.expect("HELP_PANEL not found"),
			text_help_function: text_help_function.expect("HELP_FUNCTION not found"),
			text_help_other: text_help_other.expect("HELP_OTHER not found"),
		}
	};

	// Stores the FontDefinitions used by egui
	static ref FONT_DEFINITIONS: FontDefinitions = {
		let mut font_data: BTreeMap<String, FontData> = BTreeMap::new();
		let mut families = BTreeMap::new();

		font_data.insert("Hack".to_owned(), FILE_DATA.font_hack.clone());
		font_data.insert("Ubuntu-Light".to_owned(), FILE_DATA.font_ubuntu_light.clone());
		font_data.insert("NotoEmoji-Regular".to_owned(), FILE_DATA.font_notoemoji.clone());

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

// Tests to make sure archived (and compressed) assets match expected data
#[test]
fn test_file_data() {
	assert_eq!(
		FILE_DATA.font_ubuntu_light,
		FontData::from_owned(include_bytes!("../assets/Ubuntu-Light.ttf").to_vec())
	);
	assert_eq!(
		FILE_DATA.font_notoemoji,
		FontData::from_owned(include_bytes!("../assets/NotoEmoji-Regular.ttf").to_vec())
	);
	assert_eq!(
		FILE_DATA.font_hack,
		FontData::from_owned(include_bytes!("../assets/Hack-Regular.ttf").to_vec())
	);

	assert_eq!(
		FILE_DATA.text_help_expr,
		include_str!("../assets/text_help_expr.txt")
	);
	assert_eq!(
		FILE_DATA.text_help_vars,
		include_str!("../assets/text_help_vars.txt")
	);
	assert_eq!(
		FILE_DATA.text_help_panel,
		include_str!("../assets/text_help_panel.txt")
	);
	assert_eq!(
		FILE_DATA.text_help_function,
		include_str!("../assets/text_help_function.txt")
	);
	assert_eq!(
		FILE_DATA.text_help_other,
		include_str!("../assets/text_help_other.txt")
	);
}

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::JsCast;

		// removes the "loading" element on the web page that displays a loading indicator
		fn stop_loading() {
			let document = web_sys::window().unwrap().document().unwrap();
			let loading_element = document.get_element_by_id("loading").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
			loading_element.remove();
		}
	}
}

// Used to provide info on the Licensing of the project
const LICENSE_INFO: &str = "The AGPL license ensures that the end user, even if not hosting the program itself, is still guaranteed access to the source code of the project in question.";

// The URL of the project
const PROJECT_URL: &str = "https://github.com/Titaniumtown/YTBN-Graphing-Software";

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

	pub extrema: bool,

	pub roots: bool,
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
			extrema: true,
			roots: true,
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
			functions: vec![EMPTY_FUNCTION_ENTRY.clone().integral(true)],
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

				let mut extrema_toggled: bool = false;
				let mut roots_toggled: bool = false;
				ui.horizontal(|ui| {
					extrema_toggled = ui
						.add(Button::new("Extrema"))
						.on_hover_text(match self.settings.extrema {
							true => "Disable Displaying Extrema",
							false => "Display Extrema",
						})
						.clicked();

					roots_toggled = ui
						.add(Button::new("Roots"))
						.on_hover_text(match self.settings.roots {
							true => "Disable Displaying Roots",
							false => "Display Roots",
						})
						.clicked();
				});

				self.settings.extrema.bitxor_assign(extrema_toggled);
				self.settings.roots.bitxor_assign(roots_toggled);

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

				let integral_num_changed = ui
					.add(
						Slider::new(&mut self.settings.integral_num, INTEGRAL_NUM_RANGE)
							.text("Interval"),
					)
					.changed();

				let configs_changed = max_x_changed
					| min_x_changed | integral_num_changed
					| roots_toggled | extrema_toggled;

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

					let proc_func_str = process_func_str(self.func_strs[i].clone());
					if configs_changed
						| integral_toggle | derivative_toggle
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
	fn setup(&mut self, _ctx: &Context, _frame: &Frame, _storage: Option<&dyn Storage>) {
		#[cfg(target_arch = "wasm32")]
		stop_loading();
		log_helper("egui app initialized.");
	}

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
						EMPTY_FUNCTION_ENTRY
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
					ui.label(&FILE_DATA.text_help_expr);
				});

				ui.collapsing("Supported Constants", |ui| {
					ui.label(&FILE_DATA.text_help_vars);
				});

				ui.collapsing("Panel", |ui| {
					ui.label(&FILE_DATA.text_help_panel);
				});

				ui.collapsing("Functions", |ui| {
					ui.label(&FILE_DATA.text_help_function);
				});

				ui.collapsing("Other", |ui| {
					ui.label(&FILE_DATA.text_help_other);
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

							function.display(
								plot_ui,
								minx_bounds,
								maxx_bounds,
								available_width,
								self.settings.extrema,
								self.settings.roots,
							)
						})
						.collect();
					self.last_info = (area_list, start.elapsed());
				});
		});
	}

	// Uncaps max canvas size. This was capped in egui due to a bug in Firefox. But it's fixed now.
	fn max_size_points(&self) -> Vec2 { Vec2::new(f32::MAX, f32::MAX) }
}
