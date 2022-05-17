use std::{
	collections::BTreeMap,
	env,
	fs::File,
	io::{BufWriter, Write},
	path::Path,
};

use epaint::{
	text::{FontData, FontDefinitions},
	FontFamily,
};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data.rs"));

fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets/*");

	shadow_rs::new().expect("Could not initialize shadow_rs");

	// let font_hack = FontData::from_static(include_bytes!("assets/Hack-Regular.ttf"));
	let font_ubuntu_light = FontData::from_static(include_bytes!("assets/Ubuntu-Light.ttf"));
	let font_notoemoji = FontData::from_static(include_bytes!("assets/NotoEmoji-Regular.ttf"));
	let font_emoji_icon = FontData::from_static(include_bytes!("assets/emoji-icon-font.ttf"));

	let fonts = FontDefinitions {
		font_data: BTreeMap::from([
			// ("Hack".to_owned(), font_hack),
			("Ubuntu-Light".to_owned(), font_ubuntu_light),
			("NotoEmoji-Regular".to_owned(), font_notoemoji),
			("emoji-icon-font".to_owned(), font_emoji_icon),
		]),
		families: BTreeMap::from([
			(
				FontFamily::Monospace,
				vec![
					// "Hack".to_owned(),
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

	let text_json: serde_json::Value =
		serde_json::from_str(include_str!("assets/text.json")).unwrap();
	let mut json_file_array = text_json.as_object().unwrap().clone();
	for value in json_file_array.iter_mut() {
		if let serde_json::Value::Array(values) = value.1 {
			let values_copy = values.clone();
			*value.1 = serde_json::Value::String(
				values_copy
					.iter()
					.map(|s| s.as_str().expect("failed to make a string"))
					.collect::<Vec<&str>>()
					.join("\n"),
			);
		}
	}

	let text_data: TextData = serde_json::from_value(serde_json::Value::Object(json_file_array))
		.expect("Failed to convert data to TextData");

	let data = bincode::serialize(&TotalData {
		text: text_data,
		fonts,
	})
	.unwrap();

	let zstd_levels = zstd::compression_level_range();
	let data_compressed =
		zstd::encode_all(data.as_slice(), *zstd_levels.end()).expect("Could not compress data");

	let path = Path::new(&env::var("OUT_DIR").unwrap()).join("compressed_data");
	let mut file = BufWriter::new(File::create(&path).expect("Could not save compressed_data"));

	file.write_all(data_compressed.as_slice())
		.expect("Failed to save compressed data");
}
