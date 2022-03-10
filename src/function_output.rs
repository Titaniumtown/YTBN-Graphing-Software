use eframe::{
	egui::{
		plot::{BarChart, Line, PlotUi, Points, Value, Values},
		widgets::plot::Bar,
	},
	epaint::Color32,
};

use crate::misc::digits_precision;

#[derive(Clone)]
pub struct FunctionOutput {
	pub(crate) back: Option<Vec<Value>>,
	pub(crate) integral: Option<(Vec<Bar>, f64)>,
	pub(crate) derivative: Option<Vec<Value>>,
	pub(crate) extrema: Option<Vec<Value>>,
	pub(crate) roots: Option<Vec<Value>>,
}

impl FunctionOutput {
	pub fn new_empty() -> Self {
		Self {
			back: None,
			integral: None,
			derivative: None,
			extrema: None,
			roots: None,
		}
	}

	pub fn invalidate_whole(&mut self) {
		self.back = None;
		self.integral = None;
		self.derivative = None;
		self.extrema = None;
		self.roots = None;
	}

	pub fn invalidate_back(&mut self) { self.back = None; }

	pub fn invalidate_integral(&mut self) { self.integral = None; }

	pub fn invalidate_derivative(&mut self) { self.derivative = None; }

	pub fn invalidate_points(&mut self) {
		self.extrema = None;
		self.roots = None;
	}

	pub fn display(
		&self, plot_ui: &mut PlotUi, func_str: &str, derivative_str: &str, step: f64,
		derivative_enabled: bool,
	) -> f64 {
		plot_ui.line(
			Line::new(Values::from_values(self.back.clone().unwrap()))
				.color(Color32::RED)
				.name(func_str),
		);

		if derivative_enabled {
			if let Some(derivative_data) = self.derivative.clone() {
				plot_ui.line(
					Line::new(Values::from_values(derivative_data))
						.color(Color32::GREEN)
						.name(derivative_str),
				);
			}
		}

		if let Some(extrema_data) = self.extrema.clone() {
			plot_ui.points(
				Points::new(Values::from_values(extrema_data))
					.color(Color32::YELLOW)
					.name("Extrema")
					.radius(5.0),
			);
		}

		if let Some(roots_data) = self.roots.clone() {
			plot_ui.points(
				Points::new(Values::from_values(roots_data))
					.color(Color32::LIGHT_BLUE)
					.name("Root")
					.radius(5.0),
			);
		}

		if let Some(integral_data) = self.integral.clone() {
			plot_ui.bar_chart(
				BarChart::new(integral_data.0)
					.color(Color32::BLUE)
					.width(step),
			);

			digits_precision(integral_data.1, 8)
		} else {
			f64::NAN
		}
	}
}
