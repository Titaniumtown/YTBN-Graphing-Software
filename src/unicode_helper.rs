use itertools::Itertools;

#[allow(dead_code)]
pub fn to_unicode_hash(c: char) -> String {
	c.escape_unicode()
		.to_string()
		.replace(r#"\\u{"#, "")
		.replace(['{', '}'], "")
		.to_uppercase()
}

#[allow(dead_code)]
pub fn to_chars_array(chars: Vec<char>) -> String {
	[
		"[",
		&chars
			.iter()
			.map(|c| format!("'{}'", c.escape_unicode()))
			.join(", "),
		"]",
	]
	.concat()
}
