#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextData {
	pub help_expr: String,
	pub help_vars: String,
	pub help_panel: String,
	pub help_function: String,
	pub help_other: String,
	pub license_info: String,
	pub welcome: String,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct TotalData {
	pub text: TextData,
	pub fonts: epaint::text::FontDefinitions,
}
