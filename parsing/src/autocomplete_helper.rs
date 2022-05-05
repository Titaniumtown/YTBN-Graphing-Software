use core::cmp::Ordering;
use std::collections::HashSet;

/// https://www.dotnetperls.com/sort-rust
fn compare_len_reverse_alpha(a: &String, b: &String) -> Ordering {
	match a.len().cmp(&b.len()) {
		Ordering::Equal => b.cmp(a),
		order => order,
	}
}

/// Generates hashmap (well really a vector of tuple of strings that are then turned into a hashmap by phf)
#[allow(dead_code)]
pub(crate) fn compile_hashmap(data: Vec<String>) -> Vec<(String, String)> {
	let mut seen = HashSet::new();

	let tuple_list_1: Vec<(String, String)> = data
		.iter()
		.map(|e| e.to_string() + "(")
		.flat_map(|func| all_possible_splits(func, &mut seen))
		.collect();

	let keys: Vec<&String> = tuple_list_1.iter().map(|(a, _)| a).collect();
	let mut output: Vec<(String, String)> = Vec::new();
	let mut seen_3: HashSet<String> = HashSet::new();

	for (key, value) in tuple_list_1.iter() {
		if seen_3.contains(&*key) {
			continue;
		}

		seen_3.insert(key.clone());

		let count_keys = keys.iter().filter(|a| a == &&key).count();

		if count_keys == 1 {
			output.push((key.clone(), format!(r#"Hint::Single("{}")"#, value)));
		} else if count_keys > 1 {
			let mut multi_data = tuple_list_1
				.iter()
				.filter(|(a, _)| a == key)
				.map(|(_, b)| b)
				.collect::<Vec<&String>>();
			multi_data.sort_unstable_by(|a, b| compare_len_reverse_alpha(a, b));
			output.push((key.clone(), format!("Hint::Many(&{:?})", multi_data)));
		} else {
			panic!("Number of values for {key} is 0!");
		}
	}
	output
}

/// Returns a vector of all possible splitting combinations of a strings
#[allow(dead_code)]
fn all_possible_splits(
	func: String, seen: &mut HashSet<(String, String)>,
) -> Vec<(String, String)> {
	(1..func.len())
		.map(|i| {
			let (first, last) = func.split_at(i);
			(first.to_string(), last.to_string())
		})
		.flat_map(|(first, last)| {
			if seen.contains(&(first.clone(), last.clone())) {
				return None;
			}
			seen.insert((first.to_string(), last.to_string()));

			Some((first, last))
		})
		.collect::<Vec<(String, String)>>()
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests to make sure hashmap generation works as expected
	#[test]
	fn hashmap_gen_test() {
		let data = vec!["time", "text", "test"];
		let expect = vec![
			("t", r#"Hint::Many(&["ime(", "ext(", "est("])"#),
			("ti", r#"Hint::Single("me(")"#),
			("tim", r#"Hint::Single("e(")"#),
			("time", r#"Hint::Single("(")"#),
			("te", r#"Hint::Many(&["xt(", "st("])"#),
			("tex", r#"Hint::Single("t(")"#),
			("text", r#"Hint::Single("(")"#),
			("tes", r#"Hint::Single("t(")"#),
			("test", r#"Hint::Single("(")"#),
		];

		assert_eq!(
			compile_hashmap(data.iter().map(|e| e.to_string()).collect()),
			expect
				.iter()
				.map(|(a, b)| (a.to_string(), b.to_string()))
				.collect::<Vec<(String, String)>>()
		);
	}
}
