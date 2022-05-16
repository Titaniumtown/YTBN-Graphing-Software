use ytbn_graphing_software::{AppSettings, FunctionEntry, Riemann};

fn app_settings_constructor(
	sum: Riemann, integral_min_x: f64, integral_max_x: f64, pixel_width: usize,
	integral_num: usize, min_x: f64, max_x: f64,
) -> AppSettings {
	AppSettings {
		riemann_sum: sum,
		integral_min_x,
		integral_max_x,
		min_x,
		max_x,
		integral_changed: true,
		integral_num,
		do_extrema: false,
		do_roots: false,
		plot_width: pixel_width,
	}
}

static BACK_TARGET: [(f64, f64); 11] = [
	(-1.0, 1.0),
	(-0.8, 0.6400000000000001),
	(-0.6, 0.36),
	(-0.4, 0.16000000000000003),
	(-0.19999999999999996, 0.03999999999999998),
	(0.0, 0.0),
	(0.19999999999999996, 0.03999999999999998),
	(0.3999999999999999, 0.15999999999999992),
	(0.6000000000000001, 0.3600000000000001),
	(0.8, 0.6400000000000001),
	(1.0, 1.0),
];

static DERIVATIVE_TARGET: [(f64, f64); 11] = [
	(-1.0, -2.0),
	(-0.8, -1.6),
	(-0.6, -1.2),
	(-0.4, -0.8),
	(-0.19999999999999996, -0.3999999999999999),
	(0.0, 0.0),
	(0.19999999999999996, 0.3999999999999999),
	(0.3999999999999999, 0.7999999999999998),
	(0.6000000000000001, 1.2000000000000002),
	(0.8, 1.6),
	(1.0, 2.0),
];

fn do_test(sum: Riemann, area_target: f64) {
	let settings = app_settings_constructor(sum, -1.0, 1.0, 10, 10, -1.0, 1.0);

	let mut function = FunctionEntry::EMPTY;
	function.update_string("x^2");
	function.integral = true;
	function.derivative = true;

	function.tests(
		settings,
		BACK_TARGET.to_vec(),
		DERIVATIVE_TARGET.to_vec(),
		area_target,
	);
}

#[test]
fn left_function() { do_test(Riemann::Left, 0.9600000000000001); }

#[test]
fn middle_function() { do_test(Riemann::Middle, 0.92); }

#[test]
fn right_function() { do_test(Riemann::Right, 0.8800000000000001); }
