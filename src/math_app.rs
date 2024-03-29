use crate::{
	consts::{build, BUILD_INFO, COLORS, DEFAULT_INTEGRAL_NUM, DEFAULT_MAX_X, DEFAULT_MIN_X},
	function_entry::Riemann,
	function_manager::FunctionManager,
	misc::option_vec_printer,
};
use eframe::App;
use egui::{
	style::Margin, Button, CentralPanel, Color32, ComboBox, Context, DragValue, Frame, Key, Layout,
	SidePanel, TopBottomPanel, Ui, Vec2, Window,
};
use egui_plot::Plot;

use emath::{Align, Align2};
use epaint::Rounding;
use instant::Instant;
use itertools::Itertools;
use std::{io::Read, ops::BitXorAssign};

/// Stores current settings/state of [`MathApp`]
#[derive(Copy, Clone)]
pub struct AppSettings {
	/// Stores the type of Rienmann sum that should be calculated
	pub riemann_sum: Riemann,

	/// Min and Max range for calculating an integral
	pub integral_min_x: f64,

	/// Max value for calculating an
	pub integral_max_x: f64,

	/// Minimum x bound of plot
	pub min_x: f64,

	/// Maximum x bound of plot
	pub max_x: f64,

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

impl const Default for AppSettings {
	/// Default implementation of `AppSettings`, this is how the application starts up
	fn default() -> Self {
		Self {
			riemann_sum: Riemann::default(),
			integral_min_x: DEFAULT_MIN_X,
			integral_max_x: DEFAULT_MAX_X,
			min_x: 0.0,
			max_x: 0.0,
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
	/// Help window
	pub help: bool,

	/// Info window
	pub info: bool,

	/// Sidepanel
	pub side_panel: bool,

	/// Welcome introduction
	pub welcome: bool,
}

impl const Default for Opened {
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
	functions: FunctionManager,

	/// Contains the list of Areas calculated (the vector of f64) and time it took for the last frame (the Duration). Stored in a Tuple.
	last_info: (Option<String>, Option<String>),

	/// Stores opened windows/elements for later reference
	opened: Opened,

	/// Stores settings (pretty self-explanatory)
	settings: AppSettings,
}

#[cfg(target_arch = "wasm32")]
fn get_window() -> web_sys::Window { web_sys::window().expect("Could not get web_sys window") }

#[cfg(target_arch = "wasm32")]
fn get_localstorage() -> web_sys::Storage {
	get_window()
		.local_storage()
		.expect("failed to get localstorage1")
		.expect("failed to get localstorage2")
}

#[cfg(target_arch = "wasm32")]
const DATA_NAME: &str = "YTBN-DECOMPRESSED";
#[cfg(target_arch = "wasm32")]
const FUNC_NAME: &str = "YTBN-FUNCTIONS";

impl MathApp {
	#[allow(dead_code)] // This is used lol
	/// Create new instance of [`MathApp`] and return it
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		#[cfg(threading)]
		tracing::info!("Threading: Enabled");

		#[cfg(not(threading))]
		tracing::info!("Threading: Disabled");

		tracing::info!("commit: {}", build::SHORT_COMMIT);

		tracing::info!("Initializing...");
		let start = Instant::now();

		cfg_if::cfg_if! {
			if #[cfg(target_arch = "wasm32")] {

				tracing::info!("Web Info: {:?}", &cc.integration_info.web_info);

				fn get_storage_decompressed() -> Option<Vec<u8>> {
					let data = get_localstorage().get_item(DATA_NAME).ok()??;
					let (commit, cached_data) = crate::misc::hashed_storage_read(&data)?;


					if commit == unsafe { std::mem::transmute::<&str, crate::misc::HashBytes>(build::SHORT_COMMIT) } {
						tracing::info!("Reading decompression cache. Bytes: {}", cached_data.len());
						return Some(cached_data.to_vec());
					} else {
						None
					}
				}

				fn load_functions() -> Option<FunctionManager> {
					let data = get_localstorage().get_item(FUNC_NAME).ok()??;
					if crate::misc::HASH_LENGTH >= data.len() {
						return None;
					}

					// TODO: stabilize FunctionManager serialize so it can persist across builds
					let (commit, func_data) = crate::misc::hashed_storage_read(&data)?;


					if commit == unsafe { std::mem::transmute::<&str, &[u8]>(build::SHORT_COMMIT) } {
						tracing::info!("Reading previous function data");
						let function_manager: FunctionManager = bincode::deserialize(&func_data).ok()?;
						return Some(function_manager);
					} else {
						None
					}
				}

			}
		}

		fn decompress_fonts() -> epaint::text::FontDefinitions {
			let mut data = Vec::new();
			let _ = ruzstd::StreamingDecoder::new(
				&mut const { include_bytes!(concat!(env!("OUT_DIR"), "/compressed_data")).as_slice() },
			)
			.expect("unable to decode compressed data")
			.read_to_end(&mut data)
			.expect("unable to read compressed data");

			#[cfg(target = "wasm32")]
			{
				tracing::info!("Setting decompression cache");
				let commit: crate::misc::HashBytes = const {
					unsafe {
						std::mem::transmute::<&str, crate::misc::HashBytes>(build::SHORT_COMMIT)
					}
				};
				let saved_data = commit.hashed_storage_create(data);
				tracing::info!("Bytes: {}", saved_data.len());
				get_localstorage()
					.set_item(DATA_NAME, saved_data)
					.expect("failed to set local storage cache");
			}

			bincode::deserialize(data.as_slice()).expect("unable to deserialize bincode")
		}

		tracing::info!("Reading fonts...");

		// Initialize fonts
		// This used to be in the `update` method, but (after a ton of digging) this actually caused OOMs. that was a pain to debug
		cc.egui_ctx.set_fonts({
			#[cfg(target = "wasm32")]
			if let Some(Ok(data)) =
				get_storage_decompressed().map(|data| bincode::deserialize(data.as_slice()))
			{
				data
			} else {
				decompress_fonts()
			}

			#[cfg(not(target = "wasm32"))]
			decompress_fonts()
		});

		// Set dark mode by default
		// cc.egui_ctx.set_visuals(crate::style::style());

		// Set spacing
		// cc.egui_ctx.set_spacing(crate::style::SPACING);

		tracing::info!("Initialized! Took: {:?}", start.elapsed());

		Self {
			#[cfg(target_arch = "wasm32")]
			functions: load_functions().unwrap_or_default(),

			#[cfg(not(target_arch = "wasm32"))]
			functions: FunctionManager::default(),

			last_info: (None, None),
			opened: Opened::default(),
			settings: AppSettings::default(),
		}
	}

	/// Creates SidePanel which contains configuration options
	fn side_panel(&mut self, ctx: &Context) {
		// Side Panel which contains vital options to the operation of the application
		// (such as adding functions and other options)
		SidePanel::left("side_panel")
			.resizable(false)
			.show(ctx, |ui| {
				let any_using_integral = self.functions.any_using_integral();
				let prev_sum = self.settings.riemann_sum;
				// ComboBox for selecting what Riemann sum type to use
				ui.add_enabled_ui(any_using_integral, |ui| {
					let spacing_mut = ui.spacing_mut();

					spacing_mut.item_spacing.x = 1.0;
					spacing_mut.interact_size *= 0.5;
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

					let riemann_changed = prev_sum != self.settings.riemann_sum;

					let min_x_old = self.settings.integral_min_x;
					let max_x_old = self.settings.integral_max_x;

					let (min_x_changed, max_x_changed) = ui
						.horizontal(|ui: &mut Ui| {
							// let spacing_mut = ui.spacing_mut();

							// spacing_mut.item_spacing = Vec2::new(1.0, 0.0);
							// spacing_mut.interact_size *= 0.5;

							ui.label("Integral: [");
							let min_x_changed = ui
								.add(DragValue::new(&mut self.settings.integral_min_x))
								.changed();
							ui.label(",");
							let max_x_changed = ui
								.add(DragValue::new(&mut self.settings.integral_max_x))
								.changed();
							ui.label("]");
							(min_x_changed, max_x_changed)
						})
						.inner;

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
						.horizontal(|ui| {
							let spacing_mut = ui.spacing_mut();

							spacing_mut.item_spacing.x = 1.5;
							ui.label("Interval:");
							ui.add(DragValue::new(&mut self.settings.integral_num))
								.changed()
						})
						.inner;

					if integral_num_changed {
						self.settings.integral_num = self.settings.integral_num.clamp(0, 500000);
					}

					self.settings.integral_changed = any_using_integral
						&& (max_x_changed | min_x_changed | integral_num_changed | riemann_changed);
				});

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

				if self.functions.display_entries(ui) {
					#[cfg(target_arch = "wasm32")]
					{
						tracing::info!("Saving function data");
						use crate::misc::{hashed_storage_create, HashBytes};
						let hash: HashBytes =
							unsafe { std::mem::transmute::<&str, HashBytes>(build::SHORT_COMMIT) };
						let saved_data = hashed_storage_create(
							hash,
							&bincode::serialize(&self.functions)
								.expect("unable to deserialize functions"),
						);
						// tracing::info!("Bytes: {}", saved_data.len());
						get_localstorage()
							.set_item(FUNC_NAME, &saved_data)
							.expect("failed to set local function storage");
					}
				}

				// Only render if there's enough space
				if ui.available_height() > crate::consts::FONT_SIZE {
					ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
						// Contents put in reverse order from bottom to top due to the 'buttom_up' layout

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

impl App for MathApp {
	/// Called each time the UI needs repainting.
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		// start timer
		let start = if self.opened.info {
			Some(instant::Instant::now())
		} else {
			// if disabled, clear the stored formatted time
			self.last_info.1 = None;

			None
		};

		// If keyboard input isn't being grabbed, check for key combos
		if !ctx.wants_keyboard_input() {
			// If `H` key is pressed, toggle Side Panel
			self.opened
				.side_panel
				.bitxor_assign(ctx.input_mut(|x| x.consume_key(egui::Modifiers::NONE, Key::H)));
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
					self.functions.push_empty();
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

				// Display Area and time of last frame
				if let Some(ref area) = self.last_info.0 {
					ui.label(area);
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
				ui.collapsing("Supported Expressions", |ui| {
					ui.label("abs, signum, sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, floor, round, ceil, trunc, fract, exp, sqrt, cbrt, ln, log2, log10, log");
				});

				ui.collapsing("Supported Constants", |ui| {
					ui.label("- Euler's number is supported via 'e' or 'E'\n- PI is available through 'pi' or 'π'");
				});

				ui.collapsing("Panel", |ui| {
					ui.label("- The 'Panel' button toggles if the side bar should be shown or not. This can also be accomplished by pressing the 'h' key.\n- The 'Add Function' button adds a new function to be graphed. You can then configure that function in the side panel.\n- The 'Help' button opens and closes this window!\n- The 'Info' button provides information on the build currently running.");
				});

				ui.collapsing("Functions", |ui| {
					ui.label("(From Left to Right)\n`✖` allows you to delete the selected function. Deleting a function is prevented if only 1 function exists.\n`∫` toggles integration.\n`d/dx` toggles the calculation of derivatives.\n`⚙` opens a window to tweak function options.");
				});

				ui.collapsing("Other", |ui| {
					ui.label("- Extrema (local minimums and maximums) and Roots (intersections with the x-axis) are displayed though yellow and light blue points respectively located on the graph. These can be toggled in the side panel.");
				});
			});

		// Welcome window
		if self.opened.welcome {
			let welcome_response = Window::new("Welcome")
				.anchor(Align2::CENTER_CENTER, Vec2::ZERO)
				.resizable(false)
				.collapsible(false)
				.title_bar(false)
				.show(ctx, |ui| {
					ui.label("Welcome to the (Yet-to-be-named) Graphing Software!\n\nThis project aims to provide an intuitive experience graphing mathematical functions with features such as Integration, Differentiation, Extrema, Roots, and much more! (see the Help Window for more details)");
				});

			if let Some(response) = welcome_response {
				// if user clicks off welcome window, close it
				if response.response.clicked_elsewhere() {
					self.opened.welcome = false;
				}
			}
		}

		// Window with information about the build and current commit
		Window::new("Info")
			.open(&mut self.opened.info)
			.default_pos([200.0, 200.0])
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.add(egui::Label::new(BUILD_INFO));

				if let Some(ref took) = self.last_info.1 {
					ui.label(took);
				}
			});

		// If side panel is enabled, show it.
		if self.opened.side_panel {
			self.side_panel(ctx);
		}

		// Central panel which contains the central plot (or an error created when parsing)
		CentralPanel::default()
			.frame(Frame {
				inner_margin: Margin::symmetric(0.0, 0.0),
				rounding: Rounding::ZERO,
				// fill: crate::style::STYLE.window_fill(),
				fill: Color32::from_gray(27),
				..Frame::none()
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
					.join("");

				if !errors_formatted.is_empty() {
					ui.centered_and_justified(|ui| {
						ui.heading(errors_formatted);
					});
					return;
				}

				let available_width: usize = (ui.available_width() as usize) + 1; // Used in later logic
				let width_changed = available_width != self.settings.plot_width;
				self.settings.plot_width = available_width;

				// Create and setup plot
				Plot::new("plot")
					.set_margin_fraction(Vec2::ZERO)
					.data_aspect(1.0)
					.include_y(0)
					.show(ui, |plot_ui| {
						let (min_x, max_x): (f64, f64) = {
							let bounds = plot_ui.plot_bounds();
							(bounds.min()[0], bounds.max()[0])
						};

						let min_max_changed =
							(min_x != self.settings.min_x) | (max_x != self.settings.max_x);
						let did_zoom = (max_x - min_x).abs()
							!= (self.settings.max_x - self.settings.min_x).abs();
						self.settings.min_x = min_x;
						self.settings.max_x = max_x;

						self.functions
							.get_entries_mut()
							.iter_mut()
							.for_each(|(_, function)| {
								function.calculate(
									width_changed,
									min_max_changed,
									did_zoom,
									self.settings,
								)
							});

						let area: Vec<Option<f64>> = self
							.functions
							.get_entries()
							.iter()
							.enumerate()
							.map(|(i, (_, function))| {
								function.display(plot_ui, &self.settings, COLORS[i])
							})
							.collect();

						self.last_info.0 = if area.iter().any(|e| e.is_some()) {
							Some(format!("Area: {}", option_vec_printer(area.as_slice())))
						} else {
							None
						};
					});
			});

		// Calculate and store the last time it took to draw the frame
		self.last_info.1 = start.map(|a| format!("Took: {}ms", a.elapsed().as_micros()));
	}
}
