use parsing::Hint;
use ytbn_graphing_software::{AutoComplete, Movement};

enum Action<'a> {
	AssertIndex(usize),
	AssertString(&'a str),
	AssertHint(&'a str),
	SetString(&'a str),
	Move(Movement),
}

fn ac_tester(actions: &[Action]) {
	let mut ac = AutoComplete::default();
	for action in actions.iter() {
		match action {
			Action::AssertIndex(target_i) => {
				if &ac.i != target_i {
					panic!(
						"AssertIndex failed: Current: '{}' Expected: '{}'",
						ac.i, target_i
					)
				}
			}
			Action::AssertString(target_string) => {
				if &ac.string != target_string {
					panic!(
						"AssertString failed: Current: '{}' Expected: '{}'",
						ac.string, target_string
					)
				}
			}
			Action::AssertHint(target_hint) => match ac.hint {
				Hint::None => {
					if !target_hint.is_empty() {
						panic!(
							"AssertHint failed on `Hint::None`: Expected: {}",
							target_hint
						);
					}
				}
				Hint::Many(hints) => {
					let hint = hints[ac.i];
					if &hint != target_hint {
						panic!(
								"AssertHint failed on `Hint::Many`: Current: '{}' (index: {}) Expected: '{}'",
								hint, ac.i, target_hint
							)
					}
				}
				Hint::Single(hint) => {
					if hint != target_hint {
						panic!(
							"AssertHint failed on `Hint::Single`: Current: '{}' Expected: '{}'",
							hint, target_hint
						)
					}
				}
			},
			Action::SetString(target_string) => {
				ac.update_string(target_string);
			}
			Action::Move(target_movement) => {
				ac.register_movement(target_movement);
			}
		}
	}
}

#[test]
fn single() {
	ac_tester(&[
		Action::SetString(""),
		Action::AssertHint("x^2"),
		Action::Move(Movement::Up),
		Action::AssertIndex(0),
		Action::AssertString(""),
		Action::AssertHint("x^2"),
		Action::Move(Movement::Down),
		Action::AssertIndex(0),
		Action::AssertString(""),
		Action::AssertHint("x^2"),
		Action::Move(Movement::Complete),
		Action::AssertString("x^2"),
		Action::AssertHint(""),
		Action::AssertIndex(0),
	]);
}

#[test]
fn multi() {
	ac_tester(&[
		Action::SetString("s"),
		Action::AssertHint("in("),
		Action::Move(Movement::Up),
		Action::AssertIndex(3),
		Action::AssertString("s"),
		Action::AssertHint("ignum("),
		Action::Move(Movement::Down),
		Action::AssertIndex(0),
		Action::AssertString("s"),
		Action::AssertHint("in("),
		Action::Move(Movement::Down),
		Action::AssertIndex(1),
		Action::AssertString("s"),
		Action::AssertHint("qrt("),
		Action::Move(Movement::Up),
		Action::AssertIndex(0),
		Action::AssertString("s"),
		Action::AssertHint("in("),
		Action::Move(Movement::Complete),
		Action::AssertString("sin("),
		Action::AssertHint(")"),
		Action::AssertIndex(0),
	]);
}

#[test]
fn parens() {
	ac_tester(&[
		Action::SetString("sin(x"),
		Action::AssertHint(")"),
		Action::Move(Movement::Up),
		Action::AssertIndex(0),
		Action::AssertString("sin(x"),
		Action::AssertHint(")"),
		Action::Move(Movement::Down),
		Action::AssertIndex(0),
		Action::AssertString("sin(x"),
		Action::AssertHint(")"),
		Action::Move(Movement::Complete),
		Action::AssertString("sin(x)"),
		Action::AssertHint(""),
		Action::AssertIndex(0),
	]);
}
