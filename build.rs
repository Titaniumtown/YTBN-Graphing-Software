use std::{
    collections::BTreeMap,
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::Arc,
};

use epaint::{
    FontFamily,
    text::{FontData, FontDefinitions, FontTweak},
};

use run_script::ScriptOptions;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/unicode_helper.rs"
));

fn font_stripper(from: &str, out: &str, unicodes: Vec<char>) -> Result<Vec<u8>, String> {
    let unicodes: Vec<String> = unicodes.iter().map(|c| to_unicode_hash(*c)).collect();

    let new_path = [&env::var("OUT_DIR").unwrap(), out].concat();
    let unicodes_formatted = unicodes
        .iter()
        .map(|u| format!("U+{}", u))
        .collect::<Vec<String>>()
        .join(",");

    // Test to see if pyftsubset is found
    let pyftsubset_detect = run_script::run("whereis pyftsubset", &(vec![]), &ScriptOptions::new());
    match pyftsubset_detect {
        Ok((_i, s1, _s2)) => {
            if s1 == "pyftsubset: " {
                return Err(String::from("pyftsubset not found"));
            }
        }
        // It was not, return an error and abort
        Err(x) => return Err(x.to_string()),
    }

    let script_result = run_script::run(
        &format!(
            "pyftsubset {}/assets/{} --unicodes={}
			mv {}/assets/{} {}",
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

    shadow_rs::ShadowBuilder::builder()
        .build()
        .expect("Could not initialize shadow_rs");

    let mut main_chars: Vec<char> =
		b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzsu0123456789?.,!(){}[]-_=+-/<>'\\ :^*`@#$%&|~;"
			.iter()
			.map(|c| *c as char)
			.collect();

    main_chars.append(&mut vec!['π', '"']);

    {
        let filtered_chars: Vec<char> = main_chars
            .iter()
            .filter(|c| !c.is_alphanumeric())
            .cloned()
            .collect();

        let chars_array = format!(
            "const VALID_EXTRA_CHARS: [char; {}] = {};",
            filtered_chars.len(),
            to_chars_array(filtered_chars),
        );
        let path = Path::new(&env::var("OUT_DIR").unwrap()).join("valid_chars.rs");
        let mut file = BufWriter::new(File::create(path).expect("Could not save compressed_data"));

        write!(&mut file, "{}", chars_array).expect("unable to write chars_array");
    }

    let fonts = FontDefinitions {
        font_data: BTreeMap::from([
            (
                "Ubuntu-Light".to_owned(),
                Arc::new(FontData::from_owned(
                    font_stripper(
                        "Ubuntu-Light.ttf",
                        "ubuntu-light.ttf",
                        [main_chars, vec!['∫']].concat(),
                    )
                    .unwrap(),
                )),
            ),
            (
                "NotoEmoji-Regular".to_owned(),
                Arc::new(FontData::from_owned(
                    font_stripper(
                        "NotoEmoji-Regular.ttf",
                        "noto-emoji.ttf",
                        vec!['🌞', '🌙', '✖'],
                    )
                    .unwrap(),
                )),
            ),
            (
                "emoji-icon-font".to_owned(),
                Arc::new(
                    FontData::from_owned(
                        font_stripper(
                            "emoji-icon-font.ttf",
                            "emoji-icon.ttf",
                            vec!['⚙', '⎘', '👁', '○', '⬆', '⬇', '⚠'],
                        )
                        .unwrap(),
                    )
                    .tweak(FontTweak {
                        scale: 0.8,
                        y_offset_factor: 0.07,
                        y_offset: 0.0,
                    }),
                ),
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

    let data = bincode::serialize(&fonts).unwrap();

    let zstd_levels = zstd::compression_level_range();
    let data_compressed =
        zstd::encode_all(data.as_slice(), *zstd_levels.end()).expect("Could not compress data");

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("compressed_data");
    let mut file = BufWriter::new(File::create(path).expect("Could not save compressed_data"));

    file.write_all(data_compressed.as_slice())
        .expect("Failed to save compressed data");
}
