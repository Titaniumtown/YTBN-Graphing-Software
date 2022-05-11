use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct TextData {
	pub help_expr: String,
	pub help_vars: String,
	pub help_panel: String,
	pub help_function: String,
	pub help_other: String,
	pub license_info: String,
	pub welcome: String,
}

#[derive(Serialize, Deserialize)]
pub struct TotalData {
	pub text: TextData,
	pub fonts: epaint::text::FontDefinitions,
}
