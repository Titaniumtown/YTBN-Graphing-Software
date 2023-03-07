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
		step_helper(10, 2.0, 3.0),
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

	let read = hashed_storage_read(&storage);
	assert_eq!(
		read.map(|(a, b)| (a.to_vec(), b.to_vec())),
		Some((commit.to_vec(), data.to_vec()))
	);
}

#[test]
fn invalid_hashed_storage() {
	use ytbn_graphing_software::hashed_storage_read;
	assert_eq!(hashed_storage_read("aaaa"), None);
}

// #[test]
// fn to_values() {
// 	use egui::plot::{Value, Values};
// 	use ytbn_graphing_software::EguiHelper;
// 	let data_raw = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 4.0)];
// 	let data: Vec<Value> = data_raw.iter().map(|(x, y)| Value::new(*x, *y)).collect();
// 	let values: Values = data.clone().to_values();

// 	assert_eq!(*values.get_values(), data);
// }

// #[test]
// fn to_tuple() {
// 	use egui::plot::PlotPoint;
// 	use ytbn_graphing_software::EguiHelper;
// 	let data_raw = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 4.0)];
// 	let data: Vec<Value> = data_raw
// 		.iter()
// 		.map(|(x, y)| PlotPoint::new(*x, *y))
// 		.collect();
// 	let tupled_data = data.to_tuple();

// 	assert_eq!(tupled_data, data_raw);
// }

// #[test]
// fn to_line() {
// 	use egui::plot::{Line, PlotPoint};
// 	use ytbn_graphing_software::EguiHelper;
// 	let data_raw: Vec<PlotPoint> = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 4.0)]
// 		.iter()
// 		.map(|(x, y)| PlotPoint::new(*x, *y))
// 		.collect();
// 	let data: Line = data_raw.clone().to_line();

// 	assert_eq!(*data.get_series().get_values(), data_raw);
// }

// #[test]
// fn to_points() {
// 	use egui::plot::{PlotPoint, Points};
// 	use ytbn_graphing_software::EguiHelper;
// 	let data_raw: Vec<PlotPoint> = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 4.0)]
// 		.iter()
// 		.map(|(x, y)| PlotPoint::new(*x, *y))
// 		.collect();
// 	let data: Points = data_raw.clone().to_points();

// 	assert_eq!(*data.get_series().get_values(), data_raw);
// }

#[test]
fn newtons_method() {
	use ytbn_graphing_software::newtons_method;
	let data = newtons_method(
		&|x: f64| x.powf(2.0) - 1.0,
		&|x: f64| 2.0 * x,
		3.0,
		&(0.0..5.0),
		f64::EPSILON,
	);
	assert_eq!(data, Some(1.0));

	let data = newtons_method(
		&|x: f64| x.sin(),
		&|x: f64| x.cos(),
		3.0,
		&(2.95..3.18),
		f64::EPSILON,
	);
	assert_eq!(data, Some(std::f64::consts::PI));

	let data = newtons_method(
		&|x: f64| x.sin(),
		&|_: f64| f64::NAN,
		0.0,
		&(-10.0..10.0),
		f64::EPSILON,
	);
	assert_eq!(data, None);

	let data = newtons_method(
		&|_: f64| f64::NAN,
		&|x: f64| x.sin(),
		0.0,
		&(-10.0..10.0),
		f64::EPSILON,
	);
	assert_eq!(data, None);

	let data = newtons_method(
		&|_: f64| f64::INFINITY,
		&|x: f64| x.sin(),
		0.0,
		&(-10.0..10.0),
		f64::EPSILON,
	);
	assert_eq!(data, None);

	let data = newtons_method(
		&|x: f64| x.sin(),
		&|_: f64| f64::INFINITY,
		0.0,
		&(-10.0..10.0),
		f64::EPSILON,
	);
	assert_eq!(data, None);
}

#[test]
fn to_unicode_hash() {
	use ytbn_graphing_software::to_unicode_hash;
	assert_eq!(to_unicode_hash('\u{1f31e}'), "\\U1F31E");
}

#[test]
fn to_chars_array() {
	use ytbn_graphing_software::to_chars_array;
	assert_eq!(
		to_chars_array(vec!['\u{1f31e}', '\u{2d12c}']),
		r#"['\u{1f31e}', '\u{2d12c}']"#
	);
}
