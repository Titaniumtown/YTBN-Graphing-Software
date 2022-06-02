#[allow(dead_code)]
pub fn to_unicode_hash(c: char) -> String {
	c.escape_unicode()
		.to_string()
		.replace(r#"\\u{"#, "")
		.replace('{', "")
		.replace('}', "")
		.to_uppercase()
}

#[allow(dead_code)]
pub fn to_chars_array(chars: Vec<char>) -> String {
	[
		"[",
		&chars
			.iter()
			.map(|c| format!("'{}'", c.escape_unicode()))
			.enumerate()
			.map(|(i, x)| {
				// Add comma and space if needed
				match chars.len() > i + 1 {
					true => x + ", ",
					false => x,
				}
			})
			.collect::<Vec<String>>()
			.concat(),
		"]",
	]
	.concat()
}
