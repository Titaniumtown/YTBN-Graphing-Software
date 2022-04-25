use crate::consts::*;
use crate::function_entry::Riemann;
use crate::function_manager::Manager;
use crate::misc::{dyn_mut_iter, option_vec_printer, JsonFileOutput, SerdeValueHelper};
use egui::style::Margin;
use egui::Frame;
use egui::{
	plot::Plot, vec2, Button, CentralPanel, Color32, ComboBox, Context, FontData, FontDefinitions,
	FontFamily, Key, RichText, SidePanel, Slider, TopBottomPanel, Vec2, Visuals, Window,
};
use emath::Align2;
use epaint::Rounding;
use instant::Duration;
use std::collections::BTreeMap;
use std::{io::Read, ops::BitXorAssign, str};

#[cfg(threading)]
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::JsCast;

		/// Removes the "loading" element on the web page that displays a loading indicator
		fn stop_loading() {
			let document = web_sys::window().expect("Could not get web_sys window").document().expect("Could not get web_sys document");

			let loading_element = document.get_element_by_id("loading").expect("Couldn't get loading indicator element")
			.dyn_into::<web_sys::HtmlElement>().unwrap();

			// Remove the element
			loading_element.remove();
		}
	}
}

/// Stores current settings/state of [`MathApp`]
// TODO: find a better name for this
#[derive(Copy, Clone)]
pub struct AppSettings {
	/// Stores the type of Rienmann sum that should be calculated
	pub riemann_sum: Riemann,

	/// Min and Max range for calculating an integral
	pub integral_min_x: f64,

	/// Max value for calculating an
	pub integral_max_x: f64,

	/// Stores whether or not integral settings have changed
	pub integral_changed: bool,

	/// Number of rectangles used to calculate integral
	pub integral_num: usize,

	/// Stores whether or not displaying extrema is enabled
	pub do_extrema: bool,

	/// Stores whether or not displaying roots is enabled
	pub do_roots: bool,

	/// Stores current plot pixel width
	pub plot_width: usize,
}

impl Default for AppSettings {
	/// Default implementation of `AppSettings`, this is how the application
	/// starts up
	fn default() -> Self {
		Self {
			riemann_sum: DEFAULT_RIEMANN,
			integral_min_x: DEFAULT_MIN_X,
			integral_max_x: DEFAULT_MAX_X,
			integral_changed: true,
			integral_num: DEFAULT_INTEGRAL_NUM,
			do_extrema: true,
			do_roots: true,
			plot_width: 0,
		}
	}
}

/// Used to store the opened of windows/widgets
struct Opened {
	pub help: bool,
	pub info: bool,
	pub side_panel: bool,
	pub welcome: bool,
}

impl Default for Opened {
	fn default() -> Opened {
		Self {
			help: false,
			info: false,
			side_panel: true,
			welcome: true,
		}
	}
}

/// The actual application
pub struct MathApp {
	/// Stores vector of functions
	functions: Manager,

	/// Contains the list of Areas calculated (the vector of f64) and time it took for the last frame (the Duration). Stored in a Tuple.
	last_info: (Vec<Option<f64>>, Option<Duration>),

	/// Stores whether or not dark mode is enabled
	dark_mode: bool,

	/// Stores opened windows/elements for later reference
	opened: Opened,

	text: JsonFileOutput,

	/// Stores settings (pretty self-explanatory)
	settings: AppSettings,
}

impl MathApp {
	#[allow(dead_code)] // This is used lol
	/// Create new instance of [`MathApp`] and return it
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let start = instant::Instant::now();

		// Remove loading indicator on wasm
		#[cfg(target_arch = "wasm32")]
		stop_loading();

		#[cfg(threading)]
		tracing::info!("Threading: Enabled");

		#[cfg(not(threading))]
		tracing::info!("Threading: Disabled");

		tracing::info!("Integration name: {}", cc.integration_info.name);

		if let Some(web_info) = &cc.integration_info.web_info {
			tracing::info!("Web Info: {:?}", web_info);
		}

		tracing::info!("Loading assets...");
		let mut tar_file_data = Vec::new();
		let _ = ruzstd::StreamingDecoder::new(&mut include_bytes!("../assets.tar.zst").as_slice())
			.expect("failed to decompress assets")
			.read_to_end(&mut tar_file_data)
			.expect("failed to read assets");

		let mut tar_archive = tar::Archive::new(&*tar_file_data);

		// Stores fonts
		let mut font_ubuntu_light: Option<FontData> = None;
		let mut font_notoemoji: Option<FontData> = None;
		let mut font_hack: Option<FontData> = None;
		let mut font_emoji_icon: Option<FontData> = None;

		// Stores text
		let mut text_data: Option<JsonFileOutput> = None;

		tracing::info!("Reading assets...");
		// Iterate through all entries in the tarball
		for file in tar_archive.entries().unwrap() {
			let mut file = file.unwrap();
			let mut data: Vec<u8> = Vec::new();
			file.read_to_end(&mut data).unwrap();
			let path = file.header().path().unwrap();
			let path_string = path.to_string_lossy();

			tracing::debug!("Loading file: {}", path_string);

			// Match the file extention
			if path_string.ends_with(".ttf") {
				// Parse font files
				let font_data = FontData::from_owned(data);
				match path_string.as_ref() {
					"Hack-Regular.ttf" => {
						font_hack = Some(font_data);
					}
					"NotoEmoji-Regular.ttf" => {
						font_notoemoji = Some(font_data);
					}
					"Ubuntu-Light.ttf" => {
						font_ubuntu_light = Some(font_data);
					}
					"emoji-icon-font.ttf" => {
						font_emoji_icon = Some(font_data.tweak(egui::FontTweak {
							scale: 0.8,            // make it smaller
							y_offset_factor: 0.07, // move it down slightly
							y_offset: 0.0,
						}))
					}
					_ => {
						panic!("Font File {} not expected!", path_string);
					}
				}
			} else if path_string == "text.json" {
				text_data = Some(
					SerdeValueHelper::new(str::from_utf8(&data).expect("unable to read text.json"))
						.parse_values(),
				);
			} else {
				panic!("File {} not expected!", path_string);
			}
		}

		let fonts = FontDefinitions {
			font_data: BTreeMap::from([
				(
					"Hack".to_owned(),
					font_hack.expect("Hack -Regular.ttf not found!"),
				),
				(
					"Ubuntu-Light".to_owned(),
					font_ubuntu_light.expect("Ubuntu-Light.ttf not found!"),
				),
				(
					"NotoEmoji-Regular".to_owned(),
					font_notoemoji.expect("NotoEmoji-Regular.ttf not found!"),
				),
				(
					"emoji-icon-font".to_owned(),
					font_emoji_icon.expect("emoji-icon-font.ttf not found!"),
				),
			]),
			families: BTreeMap::from([
				(
					FontFamily::Monospace,
					vec![
						"Hack".to_owned(),
						"Ubuntu-Light".to_owned(),
						"NotoEmoji-Regular".to_owned(),
						"emoji-icon-font".to_owned(),
					],
				),
				(
					FontFamily::Proportional,
					vec![
						"Ubuntu-Light".to_owned(),
						"NotoEmoji-Regular".to_owned(),
						"emoji-icon-font".to_owned(),
					],
				),
			]),
		};

		// Initialize fonts
		// this used to be in the `update` method, but (after a ton of digging) this actually caused OOMs. that was a pain to debug
		cc.egui_ctx.set_fonts(fonts);

		tracing::info!("Initialized! Took: {:?}", start.elapsed());

		Self {
			functions: Default::default(),
			last_info: (vec![None], None),
			dark_mode: true,
			text: text_data.expect("text.json failed to load"),
			opened: Opened::default(),
			settings: Default::default(),
		}
	}

	/// Creates SidePanel which contains configuration options
	fn side_panel(&mut self, ctx: &Context) {
		// Side Panel which contains vital options to the operation of the application
		// (such as adding functions and other options)
		SidePanel::left("side_panel")
			.resizable(false)
			.show(ctx, |ui| {
				let prev_sum = self.settings.riemann_sum;
				// ComboBox for selecting what Riemann sum type to use
				ui.add_enabled_ui(self.functions.any_using_integral(), |ui| {
					ComboBox::from_label("Riemann Sum")
						.selected_text(self.settings.riemann_sum.to_string())
						.show_ui(ui, |ui| {
							ui.selectable_value(
								&mut self.settings.riemann_sum,
								Riemann::Left,
								"Left",
							);
							ui.selectable_value(
								&mut self.settings.riemann_sum,
								Riemann::Middle,
								"Middle",
							);
							ui.selectable_value(
								&mut self.settings.riemann_sum,
								Riemann::Right,
								"Right",
							);
						});
				});

				let riemann_changed = prev_sum != self.settings.riemann_sum;

				ui.horizontal(|ui| {
					self.settings.do_extrema.bitxor_assign(
						ui.add(Button::new("Extrema"))
							.on_hover_text(match self.settings.do_extrema {
								true => "Disable Displaying Extrema",
								false => "Display Extrema",
							})
							.clicked(),
					);

					self.settings.do_roots.bitxor_assign(
						ui.add(Button::new("Roots"))
							.on_hover_text(match self.settings.do_roots {
								true => "Disable Displaying Roots",
								false => "Display Roots",
							})
							.clicked(),
					);
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

				// Checks integral bounds, and if they are invalid, fix them
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

				// Number of Rectangles for Riemann sum
				let integral_num_changed = ui
					.add(
						Slider::new(&mut self.settings.integral_num, INTEGRAL_NUM_RANGE)
							.text("Interval"),
					)
					.changed();

				self.settings.integral_changed =
					max_x_changed | min_x_changed | integral_num_changed | riemann_changed;

				self.functions.display_entries(ui);

				// Only render if there's enough space
				if ui.available_height() > 0.0 {
					ui.with_layout(egui::Layout::bottom_up(emath::Align::Min), |ui| {
						// Contents put in reverse order from bottom to top due to the 'buttom_up' layout

						// Licensing information
						ui.label(
							RichText::new("(and licensed under AGPLv3)").color(Color32::LIGHT_GRAY),
						)
						.on_hover_text(&self.text.license_info);

						// Hyperlink to project's github
						ui.hyperlink_to(
							"I'm Open Source!",
							"https://github.com/Titaniumtown/YTBN-Graphing-Software",
						);
					});
				}
			});
	}
}

impl epi::App for MathApp {
	/// Called each time the UI needs repainting.
	fn update(&mut self, ctx: &Context, _frame: &mut epi::Frame) {
		// start timer
		let start = if self.opened.info {
			Some(instant::Instant::now())
		} else {
			None
		};

		// Set dark/light mode depending on the variable `self.dark_mode`
		ctx.set_visuals(match self.dark_mode {
			true => Visuals::dark(),
			false => Visuals::light(),
		});

		// If keyboard input isn't being grabbed, check for key combos
		if !ctx.wants_keyboard_input() {
			// If `H` key is pressed, toggle Side Panel
			self.opened
				.side_panel
				.bitxor_assign(ctx.input_mut().consume_key(egui::Modifiers::NONE, Key::H));
		}

		// Creates Top bar that contains some general options
		TopBottomPanel::top("top_bar").show(ctx, |ui| {
			ui.horizontal(|ui| {
				// Button in top bar to toggle showing the side panel
				self.opened.side_panel.bitxor_assign(
					ui.add(Button::new("Panel"))
						.on_hover_text(match self.opened.side_panel {
							true => "Hide Side Panel",
							false => "Show Side Panel",
						})
						.clicked(),
				);

				// Button to add a new function
				if ui
					.add_enabled(
						COLORS.len() > self.functions.len(),
						Button::new("Add Function"),
					)
					.on_hover_text("Create and graph new function")
					.clicked()
				{
					self.functions.new_function();
				}

				// Toggles opening the Help window
				self.opened.help.bitxor_assign(
					ui.add(Button::new("Help"))
						.on_hover_text(match self.opened.help {
							true => "Close Help Window",
							false => "Open Help Window",
						})
						.clicked(),
				);

				// Toggles opening the Info window
				self.opened.info.bitxor_assign(
					ui.add(Button::new("Info"))
						.on_hover_text(match self.opened.info {
							true => "Close Info Window",
							false => "Open Info Window",
						})
						.clicked(),
				);

				// Toggles dark/light mode
				self.dark_mode.bitxor_assign(
					ui.add(Button::new(match self.dark_mode {
						true => "🌞",
						false => "🌙",
					}))
					.on_hover_text(match self.dark_mode {
						true => "Turn the Lights on!",
						false => "Turn the Lights off.",
					})
					.clicked(),
				);

				// Display Area and time of last frame
				if self.last_info.0.iter().filter(|e| e.is_some()).count() > 0 {
					ui.label(format!("Area: {}", option_vec_printer(&self.last_info.0)));
				}
			});
		});

		// Help window with information for users
		Window::new("Help")
			.open(&mut self.opened.help)
			.default_pos([200.0, 200.0])
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.heading("Help With...");

				ui.collapsing("Supported Expressions", |ui| {
					ui.label(&self.text.help_expr);
				});

				ui.collapsing("Supported Constants", |ui| {
					ui.label(&self.text.help_vars);
				});

				ui.collapsing("Panel", |ui| {
					ui.label(&self.text.help_panel);
				});

				ui.collapsing("Functions", |ui| {
					ui.label(&self.text.help_function);
				});

				ui.collapsing("Other", |ui| {
					ui.label(&self.text.help_other);
				});
			});

		// Welcome window
		Window::new("Welcome!")
			.open(&mut self.opened.welcome)
			.anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.label(&self.text.welcome);
			});

		// Window with information about the build and current commit
		Window::new("Info")
			.open(&mut self.opened.info)
			.default_pos([200.0, 200.0])
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.label(&*BUILD_INFO);

				if let Some(took) = self.last_info.1 {
					ui.label(format!("Took: {:?}", took));
				}
			});

		// If side panel is enabled, show it.
		if self.opened.side_panel {
			self.side_panel(ctx);
		}

		// Referenced in plotting code, but needs to be here so it can be later  referenced when storing `last_info`
		let mut area_list: Vec<Option<f64>> = Vec::new();

		// Central panel which contains the central plot (or an error created when
		// parsing)
		CentralPanel::default()
			.frame(Frame {
				inner_margin: Margin::symmetric(0.0, 0.0),
				rounding: Rounding::none(),
				fill: ctx.style().visuals.window_fill(),
				stroke: Default::default(),
				..Default::default()
			})
			.show(ctx, |ui| {
				// Display an error if it exists
				let errors_formatted: String = self
					.functions
					.get_entries()
					.iter()
					.map(|(_, func)| func.get_test_result())
					.enumerate()
					.filter(|(_, error)| error.is_some())
					.map(|(i, error)| {
						// use unwrap_unchecked as None Errors are already filtered out
						unsafe {
							format!("(Function #{}) {}\n", i, error.as_ref().unwrap_unchecked())
						}
					})
					.collect::<String>();

				if !errors_formatted.is_empty() {
					ui.centered_and_justified(|ui| {
						ui.heading(errors_formatted);
					});
					return;
				}

				let available_width: usize = (ui.available_width() as usize) + 1; // Used in later logic
				let width_changed = available_width != self.settings.plot_width;

				if width_changed {
					self.settings.plot_width = available_width;
				}

				// Create and setup plot
				Plot::new("plot")
					.set_margin_fraction(Vec2::ZERO)
					.data_aspect(1.0)
					.include_y(0)
					.legend(egui::plot::Legend::default())
					.show(ui, |plot_ui| {
						let bounds = plot_ui.plot_bounds();
						let minx_bounds: f64 = bounds.min()[0];
						let maxx_bounds: f64 = bounds.max()[0];

						dyn_mut_iter(self.functions.get_entries_mut()).for_each(|(_, function)| {
							function.calculate(
								&minx_bounds,
								&maxx_bounds,
								width_changed,
								&self.settings,
							)
						});

						area_list = self
							.functions
							.get_entries()
							.iter()
							.enumerate()
							.map(|(i, (_, function))| {
								function.display(plot_ui, &self.settings, COLORS[i])
							})
							.collect();
					});
			});
		// Store list of functions' areas along with the time it took to process.
		self.last_info = (area_list, start.map(|a| a.elapsed()));
	}
}
