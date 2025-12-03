use ytbn_graphing_software::{AppSettings, EguiHelper, FunctionEntry, Riemann};

fn app_settings_constructor(
    sum: Riemann,
    integral_min_x: f64,
    integral_max_x: f64,
    pixel_width: usize,
    integral_num: usize,
    min_x: f64,
    max_x: f64,
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
        do_intersections: false,
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

#[cfg(test)]
fn do_test(sum: Riemann, area_target: f64) {
    let settings = app_settings_constructor(sum, -1.0, 1.0, 10, 10, -1.0, 1.0);

    let mut function = FunctionEntry::default();
    function.update_string("x^2");
    function.integral = true;
    function.derivative = true;

    let mut settings = settings;
    {
        function.calculate(true, true, false, settings);
        assert!(!function.back_data.is_empty());
        assert_eq!(function.back_data.len(), settings.plot_width + 1);

        assert!(function.integral);
        assert!(function.derivative);

        assert_eq!(!function.root_data.is_empty(), settings.do_roots);
        assert_eq!(!function.extrema_data.is_empty(), settings.do_extrema);
        assert!(!function.derivative_data.is_empty());
        assert!(function.integral_data.is_some());

        assert_eq!(function.integral_data.clone().unwrap().1, area_target);

        let a = function.derivative_data.clone().to_tuple();

        assert_eq!(a.len(), DERIVATIVE_TARGET.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a[i].0 as f32, DERIVATIVE_TARGET[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a[i].1 as f32, DERIVATIVE_TARGET[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a, DERIVATIVE_TARGET);
            }
        }

        let a_1 = function.back_data.clone().to_tuple();

        assert_eq!(a_1.len(), BACK_TARGET.len());

        assert_eq!(a.len(), BACK_TARGET.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a_1[i].0 as f32, BACK_TARGET[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a_1[i].1 as f32, BACK_TARGET[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a_1, BACK_TARGET);
            }
        }
    }

    {
        settings.min_x += 1.0;
        settings.max_x += 1.0;
        function.calculate(true, true, false, settings);

        let a = function
            .derivative_data
            .clone()
            .to_tuple()
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        let b = DERIVATIVE_TARGET
            .iter()
            .rev()
            .take(6)
            .rev()
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        assert_eq!(a.len(), b.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a[i].0 as f32, b[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a[i].1 as f32, b[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a, b);
            }
        }

        let a_1 = function
            .back_data
            .clone()
            .to_tuple()
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        let b_1 = BACK_TARGET
            .iter()
            .rev()
            .take(6)
            .rev()
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        assert_eq!(a_1.len(), b_1.len());

        assert_eq!(a.len(), b_1.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a_1[i].0 as f32, b_1[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a_1[i].1 as f32, b_1[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a_1, b_1);
            }
        }
    }

    {
        settings.min_x -= 2.0;
        settings.max_x -= 2.0;
        function.calculate(true, true, false, settings);

        let a = function
            .derivative_data
            .clone()
            .to_tuple()
            .iter()
            .rev()
            .take(6)
            .rev()
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        let b = DERIVATIVE_TARGET
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        assert_eq!(a.len(), b.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a[i].0 as f32, b[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a[i].1 as f32, b[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a, b);
            }
        }

        let a_1 = function
            .back_data
            .clone()
            .to_tuple()
            .iter()
            .rev()
            .take(6)
            .rev()
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        let b_1 = BACK_TARGET
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<(f64, f64)>>();

        assert_eq!(a_1.len(), b_1.len());

        assert_eq!(a.len(), b_1.len());

        for i in 0..a.len() {
            if !emath::almost_equal(a_1[i].0 as f32, b_1[i].0 as f32, f32::EPSILON)
                | !emath::almost_equal(a_1[i].1 as f32, b_1[i].1 as f32, f32::EPSILON)
            {
                panic!("Expected: {:?}\nGot: {:?}", a_1, b_1);
            }
        }
    }

    {
        function.update_string("sin(x)");
        assert!(function.get_test_result().is_none());
        assert_eq!(&function.raw_func_str, "sin(x)");

        function.integral = false;
        function.derivative = false;

        assert!(!function.integral);
        assert!(!function.derivative);

        assert!(function.back_data.is_empty());
        assert!(function.integral_data.is_none());
        assert!(function.root_data.is_empty());
        assert!(function.extrema_data.is_empty());
        assert!(function.derivative_data.is_empty());

        settings.min_x -= 1.0;
        settings.max_x -= 1.0;

        function.calculate(true, true, false, settings);

        assert!(!function.back_data.is_empty());
        assert!(function.integral_data.is_none());
        assert!(function.root_data.is_empty());
        assert!(function.extrema_data.is_empty());
        assert!(!function.derivative_data.is_empty());
    }
}

#[test]
fn left_function() {
    do_test(Riemann::Left, 0.9600000000000001);
}

#[test]
fn middle_function() {
    do_test(Riemann::Middle, 0.92);
}

#[test]
fn right_function() {
    do_test(Riemann::Right, 0.8800000000000001);
}
