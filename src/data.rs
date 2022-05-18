#[derive(PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextData {
	pub help_expr: egui::RichText,
	pub help_vars: egui::RichText,
	pub help_panel: egui::RichText,
	pub help_function: egui::RichText,
	pub help_other: egui::RichText,
	pub license_info: egui::RichText,
	pub welcome: egui::RichText,
}

#[derive(PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextDataRaw {
	pub help_expr: String,
	pub help_vars: String,
	pub help_panel: String,
	pub help_function: String,
	pub help_other: String,
	pub license_info: String,
	pub welcome: String,
}

impl TextDataRaw {
	#[allow(dead_code)]
	fn into_rich(self) -> TextData {
		const SIZE: f32 = 14.0;
		use egui::RichText;
		TextData {
			help_expr: RichText::from(self.help_expr),
			help_vars: RichText::from(self.help_vars),
			help_panel: RichText::from(self.help_panel),
			help_function: RichText::from(self.help_function),
			help_other: RichText::from(self.help_other),
			license_info: RichText::from(self.license_info).size(SIZE * 0.95),
			welcome: RichText::from(self.welcome).size(SIZE + 2.0),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TotalData {
	pub text: TextData,
	pub fonts: epaint::text::FontDefinitions,
}
