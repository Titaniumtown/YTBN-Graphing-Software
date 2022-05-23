use egui::{
	style::{Margin, Selection, Spacing, WidgetVisuals, Widgets},
	Visuals,
};
use emath::vec2;
use epaint::{Color32, Rounding, Shadow, Stroke};

const fn widgets_dark() -> Widgets {
	Widgets {
		noninteractive: WidgetVisuals {
			bg_fill: Color32::from_gray(27), // window background
			bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines, windows outlines
			fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
			rounding: Rounding::same(2.0),
			expansion: 0.0,
		},
		inactive: WidgetVisuals {
			bg_fill: Color32::from_gray(60), // button background
			bg_stroke: Stroke::default(),
			fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
			rounding: Rounding::same(2.0),
			expansion: 0.0,
		},
		hovered: WidgetVisuals {
			bg_fill: Color32::from_gray(70),
			bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
			fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
			rounding: Rounding::same(3.0),
			expansion: 1.0,
		},
		active: WidgetVisuals {
			bg_fill: Color32::from_gray(55),
			bg_stroke: Stroke::new(1.0, Color32::WHITE),
			fg_stroke: Stroke::new(2.0, Color32::WHITE),
			rounding: Rounding::same(2.0),
			expansion: 1.0,
		},
		open: WidgetVisuals {
			bg_fill: Color32::from_gray(27),
			bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
			fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
			rounding: Rounding::same(2.0),
			expansion: 0.0,
		},
	}
}

pub const STYLE: Visuals = dark();
pub const SPACING: Spacing = spacing();

const fn dark() -> Visuals {
	Visuals {
		dark_mode: true,
		override_text_color: None,
		widgets: widgets_dark(),
		selection: Selection::default(),
		hyperlink_color: Color32::from_rgb(90, 170, 255),
		faint_bg_color: Color32::from_gray(35),
		extreme_bg_color: Color32::from_gray(10), // e.g. TextEdit background
		code_bg_color: Color32::from_gray(64),
		window_rounding: Rounding::same(1.5),
		window_shadow: Shadow::default(), // no shadow
		popup_shadow: Shadow::default(),  // no shadow
		resize_corner_size: 12.0,
		text_cursor_width: 2.0,
		text_cursor_preview: false,
		clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
		button_frame: true,
		collapsing_header_frame: false,
	}
}

const fn spacing() -> Spacing {
	Spacing {
		item_spacing: vec2(8.0, 3.0),
		window_margin: Margin::same(6.0),
		button_padding: vec2(4.0, 1.0),
		indent: 18.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
		interact_size: vec2(40.0, 18.0),
		slider_width: 100.0,
		text_edit_width: 280.0,
		icon_width: 14.0,
		icon_width_inner: 8.0,
		icon_spacing: 4.0,
		tooltip_width: 600.0,
		combo_height: 200.0,
		scroll_bar_width: 8.0,
		indent_ends_with_horizontal_line: false,
	}
}
