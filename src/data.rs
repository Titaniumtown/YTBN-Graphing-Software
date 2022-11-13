pub const FONT_SIZE: f32 = 14.0;

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TotalData {
	pub fonts: epaint::text::FontDefinitions,
}
