/// Tests [`SteppedVector`] to ensure everything works properly (helped me find a bunch of issues)
#[test]
fn stepped_vector() {
	use ytbn_graphing_software::SteppedVector;

	let min: i32 = -1000;
	let max: i32 = 1000;
	let data: Vec<f64> = (min..=max).map(|x| x as f64).collect();
	let len_data = data.len();
	let stepped_vector: SteppedVector = SteppedVector::from(data.as_slice());

	assert_eq!(*stepped_vector.get_min(), min as f64);
	assert_eq!(*stepped_vector.get_max(), max as f64);

	assert_eq!(stepped_vector.get_index(min as f64), Some(0));
	assert_eq!(stepped_vector.get_index(max as f64), Some(len_data - 1));

	for i in min..=max {
		assert_eq!(
			stepped_vector.get_index(i as f64),
			Some((i + min.abs()) as usize)
		);
	}

	assert_eq!(stepped_vector.get_index((min - 1) as f64), None);
	assert_eq!(stepped_vector.get_index((max + 1) as f64), None);
}

/*
/// Ensures [`decimal_round`] returns correct values
#[test]
fn decimal_round() {
	use ytbn_graphing_software::decimal_round;

	assert_eq!(decimal_round(0.00001, 1), 0.0);
	assert_eq!(decimal_round(0.00001, 2), 0.0);
	assert_eq!(decimal_round(0.00001, 3), 0.0);
	assert_eq!(decimal_round(0.00001, 4), 0.0);
	assert_eq!(decimal_round(0.00001, 5), 0.00001);

	assert_eq!(decimal_round(0.12345, 1), 0.1);
	assert_eq!(decimal_round(0.12345, 2), 0.12);
	assert_eq!(decimal_round(0.12345, 3), 0.123);
	assert_eq!(decimal_round(0.12345, 4), 0.1235); // rounds up
	assert_eq!(decimal_round(0.12345, 5), 0.12345);

	assert_eq!(decimal_round(1.9, 0), 2.0);
	assert_eq!(decimal_round(1.9, 1), 1.9);
}
*/

#[test]
fn step_helper() {
	use ytbn_graphing_software::step_helper;

	assert_eq!(
		step_helper(10, &2.0, &3.0),
		vec![2.0, 5.0, 8.0, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0, 29.0]
	);
}

/// Tests [`option_vec_printer`]
#[test]
fn option_vec_printer() {
	use std::collections::HashMap;
	use ytbn_graphing_software::option_vec_printer;

	let values_strings: HashMap<Vec<Option<&str>>, &str> = HashMap::from([
		(vec![None], "[None]"),
		(vec![Some("text"), None], "[text, None]"),
		(vec![None, None], "[None, None]"),
		(vec![Some("text1"), Some("text2")], "[text1, text2]"),
	]);

	for (key, value) in values_strings {
		assert_eq!(option_vec_printer(&key), value);
	}

	let values_nums = HashMap::from([
		(vec![Some(10)], "[10]"),
		(vec![Some(10), None], "[10, None]"),
		(vec![None, Some(10)], "[None, 10]"),
		(vec![Some(10), Some(100)], "[10, 100]"),
	]);

	for (key, value) in values_nums {
		assert_eq!(option_vec_printer(&key), value);
	}
}

#[test]
fn hashed_storage() {
	use ytbn_graphing_software::{hashed_storage_create, hashed_storage_read};

	let commit = "abcdefeg".chars().map(|c| c as u8).collect::<Vec<u8>>();
	let data = "really cool data"
		.chars()
		.map(|c| c as u8)
		.collect::<Vec<u8>>();
	let storage = hashed_storage_create(
		commit
			.as_slice()
			.try_into()
			.expect("cannot turn into [u8; 8]"),
		data.as_slice(),
	);

	let read = hashed_storage_read(storage);
	assert_eq!(read.map(|(a, b)| (a.to_vec(), b)), Some((commit, data)));
}

#[test]
fn invalid_hashed_storage() {
	use ytbn_graphing_software::hashed_storage_read;
	assert_eq!(hashed_storage_read(String::from("aaaa")), None);
}
