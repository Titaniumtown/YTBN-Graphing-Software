use std::collections::HashSet;

#[allow(dead_code)]
pub fn compile_hashmap(data: Vec<String>) -> Vec<(String, String)> {
	let start = std::time::Instant::now();
	println!("compile_hashmap");

	let functions_processed: Vec<String> = data.iter().map(|e| e.to_string() + "(").collect();

	let mut seen = HashSet::new();

	let tuple_list_1: Vec<(String, String)> = functions_processed
		.into_iter()
		.map(|func| all_possible_splits(func, &mut seen))
		.flatten()
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
			output.push((
				key.clone(),
				format!("HintEnum::Single({}{}{})", '"', value, '"'),
			));
		} else {
			let multi_data = tuple_list_1
				.iter()
				.filter(|(a, _)| a == key)
				.map(|(_, b)| b)
				.collect::<Vec<&String>>();
			output.push((key.clone(), format!("HintEnum::Many(&{:?})", multi_data)));
		}
	}
	println!("Done! {:?}", start.elapsed());

	output
}

#[allow(dead_code)]
fn all_possible_splits(
	func: String, seen: &mut HashSet<(String, String)>,
) -> Vec<(String, String)> {
	return (1..func.len())
		.map(|i| {
			let (first, last) = func.split_at(i);
			return (first.to_string(), last.to_string());
		})
		.map(|(first, last)| {
			if seen.contains(&(first.clone(), last.clone())) {
				return None;
			}
			seen.insert((first.to_string(), last.to_string()));

			return Some((first.to_string(), last.to_string()));
		})
		.filter(|a| a.is_some())
		.map(|a| a.unwrap())
		.collect::<Vec<(String, String)>>();
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
