use evalexpr::*;

pub const EPSILON: f64 = 5.0e-7;
pub const DOUBLE_EPSILON: f64 = 10.0e-7;

pub fn test_func(func_str: &str) -> Option<String> {
    let precompiled = build_operator_tree(&("x = 10;".to_owned() + func_str));

    match precompiled {
        Ok(_) => {
            let precompiled2 = precompiled.unwrap().eval();
            match precompiled2 {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            }
        }
        Err(e) => Some(e.to_string()),
    }
}

pub struct BackingFunction {
    node: Node,
}

impl BackingFunction {
    pub fn new(func_str: &str) -> Result<Self, String> {
        let precompiled = build_operator_tree(func_str);

        match precompiled {
            Ok(_) => Ok(Self {
                node: precompiled.unwrap(),
            }),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get(&self, _x: f64) -> f64 {
        let context: HashMapContext = context_map! {
            "x" => 0,
            "e" => std::f64::consts::E,
            "pi" => std::f64::consts::PI,
        }
        .unwrap();

        self.node
            .eval_with_context(&context)
            .unwrap()
            .as_number()
            .unwrap()
    }

    pub fn derivative(&self, x: f64, n: u64) -> f64 {
        if n == 0 {
            return self.get(x);
        }

        let (x1, x2) = (x - EPSILON, x + EPSILON);
        let (y1, y2) = (self.derivative(x1, n - 1), (self.derivative(x2, n - 1)));
        (y2 - y1) / DOUBLE_EPSILON
    }
}
