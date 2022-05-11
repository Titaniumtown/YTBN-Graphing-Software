use std::{
	collections::BTreeMap,
	fs::File,
	io::{BufWriter, Write},
};

use epaint::{
	text::{FontData, FontDefinitions},
	FontFamily,
};

fn main() {
	// rebuild if new commit or contents of `assets` folder changed
	println!("cargo:rerun-if-changed=.git/logs/HEAD");
	println!("cargo:rerun-if-changed=assets/*");

	shadow_rs::new().expect("Could not initialize shadow_rs");

	let font_hack = FontData::from_static(include_bytes!("assets/Hack-Regular.ttf"));
	let font_ubuntu_light = FontData::from_static(include_bytes!("assets/Ubuntu-Light.ttf"));
	let font_notoemoji = FontData::from_static(include_bytes!("assets/NotoEmoji-Regular.ttf"));
	let font_emoji_icon = FontData::from_static(include_bytes!("assets/emoji-icon-font.ttf"));

	let fonts = FontDefinitions {
		font_data: BTreeMap::from([
			("Hack".to_owned(), font_hack),
			("Ubuntu-Light".to_owned(), font_ubuntu_light),
			("NotoEmoji-Regular".to_owned(), font_notoemoji),
			("emoji-icon-font".to_owned(), font_emoji_icon),
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

	let path = "./assets/font_data";

	let fonts_data = bincode::serialize(&fonts).unwrap();
	// ::to_string(&fonts).expect("Failed to serialize fonts");

	let mut file = BufWriter::new(File::create(&path).expect("Could not create file"));

	file.write_all(fonts_data.as_slice()).unwrap();

	let _ = command_run::Command::with_args("./pack_assets.sh", &[&path])
		.enable_capture()
		.run();
}
