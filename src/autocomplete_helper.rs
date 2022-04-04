use core::cmp::Ordering;
use std::collections::HashSet;

/// https://www.dotnetperls.com/sort-rust
fn compare_len_reverse_alpha(a: &String, b: &String) -> Ordering {
	// Sort by length from short to long first.
	let length_test = a.len().cmp(&b.len());
	if length_test == Ordering::Equal {
		// If same length, sort in reverse alphabetical order.
		return b.cmp(&a);
	}
	return length_test;
}

/// Generates hashmap (well really a vector of tuple of strings that are then
/// turned into a hashmap by phf)
#[allow(dead_code)]
pub fn compile_hashmap(data: Vec<String>) -> Vec<(String, String)> {
	let start = std::time::Instant::now();
	println!("compile_hashmap");

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
		if keys.iter().filter(|a| a == &&key).count() == 1 {
			output.push((key.clone(), format!(r#"HintEnum::Single("{}")"#, value)));
		} else {
			let mut multi_data = tuple_list_1
				.iter()
				.filter(|(a, _)| a == key)
				.map(|(_, b)| b)
				.collect::<Vec<&String>>();
			multi_data.sort_unstable_by(|a, b| compare_len_reverse_alpha(a, b));
			output.push((key.clone(), format!("HintEnum::Many(&{:?})", multi_data)));
		}
	}
	println!("Done! {:?}", start.elapsed());

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

	#[test]
	fn hashmap_gen_test() {
		let data = vec!["time", "text", "test"];
		let expect = vec![
			("t", r#"HintEnum::Many(&["ime(", "ext(", "est("])"#),
			("ti", r#"HintEnum::Single("me(")"#),
			("tim", r#"HintEnum::Single("e(")"#),
			("time", r#"HintEnum::Single("(")"#),
			("te", r#"HintEnum::Many(&["xt(", "st("])"#),
			("tex", r#"HintEnum::Single("t(")"#),
			("text", r#"HintEnum::Single("(")"#),
			("tes", r#"HintEnum::Single("t(")"#),
			("test", r#"HintEnum::Single("(")"#),
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
