use std::{
	collections::BTreeMap,
	env,
	fs::File,
	io::{BufWriter, Write},
	path::Path,
};

use epaint::{
	text::{FontData, FontDefinitions, FontTweak},
	FontFamily,
};

use run_script::ScriptOptions;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data.rs"));

fn font_stripper(from: &str, out: &str, unicodes: &[&str]) -> Result<Vec<u8>, String> {
	let new_path = [&env::var("OUT_DIR").unwrap(), out].concat();
	let unicodes_formatted = unicodes
		.iter()
		.map(|u| format!("U+{}", u))
		.collect::<Vec<String>>()
		.join(",");

	let script_result = run_script::run(
		&format!(
			"
			pyftsubset {}/assets/{} --unicodes={}
			mv {}/assets/{} {}
         ",
			env!("CARGO_MANIFEST_DIR"),
			from,
			unicodes_formatted,
			env!("CARGO_MANIFEST_DIR"),
			from.replace(".ttf", ".subset.ttf"),
			new_path
		),
		&(vec![]),
		&ScriptOptions::new(),
	);

	if let Ok((_, _, error)) = script_result {
		if error.is_empty() {
			return Ok(std::fs::read(new_path).unwrap());
		} else {
			return Err(error);
		}
	} else if let Err(error) = script_result {
		return Err(error.to_string());
	}
	unreachable!()
}

fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets/*");

	shadow_rs::new().expect("Could not initialize shadow_rs");

	let font_ubuntu_light = FontData::from_static(include_bytes!("assets/Ubuntu-Light.ttf"));

	let fonts = FontDefinitions {
		font_data: BTreeMap::from([
			("Ubuntu-Light".to_owned(), font_ubuntu_light),
			(
				"NotoEmoji-Regular".to_owned(),
				FontData::from_owned(
					font_stripper(
						"NotoEmoji-Regular.ttf",
						"noto-emoji.ttf",
						&["1F31E", "1F319", "2716"],
					)
					.unwrap(),
				),
			),
			(
				"emoji-icon-font".to_owned(),
				FontData::from_owned(
					font_stripper("emoji-icon-font.ttf", "emoji-icon.ttf", &["2699"]).unwrap(),
				)
				.tweak(FontTweak {
					scale: 0.8,
					y_offset_factor: 0.07,
					y_offset: 0.0,
				}),
			),
		]),
		families: BTreeMap::from([
			(
				FontFamily::Monospace,
				vec![
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

	let text_data: TextDataRaw = serde_json::from_value(serde_json::Value::Object(json_file_array))
		.expect("Failed to convert data to TextDataRaw");

	let data = bincode::serialize(&TotalData {
		text: text_data.into_rich(),
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
