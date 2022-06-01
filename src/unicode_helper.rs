#[allow(dead_code)]
pub fn to_unicode_hash(c: char) -> String {
	c.escape_unicode()
		.to_string()
		.replace(r#"\\u{"#, "")
		.replace('{', "")
		.replace('}', "")
		.to_uppercase()
}
