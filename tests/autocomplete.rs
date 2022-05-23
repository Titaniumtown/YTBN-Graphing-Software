use parsing::{AutoComplete, Hint, Movement};

enum Action<'a> {
	AssertIndex(usize),
	AssertString(&'a str),
	AssertHint(&'a str),
	SetString(&'a str),
	Move(Movement),
}
use Action::*;

fn ac_tester(actions: &[Action]) {
	let mut ac = AutoComplete::default();
	for action in actions.iter() {
		match action {
			AssertIndex(target_i) => {
				if &ac.i != target_i {
					panic!(
						"AssertIndex failed: Current: '{}' Expected: '{}'",
						ac.i, target_i
					)
				}
			}
			AssertString(target_string) => {
				if &ac.string != target_string {
					panic!(
						"AssertString failed: Current: '{}' Expected: '{}'",
						ac.string, target_string
					)
				}
			}
			AssertHint(target_hint) => match ac.hint {
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
			SetString(target_string) => {
				ac.update_string(target_string);
			}
			Move(target_movement) => {
				ac.register_movement(target_movement);
			}
		}
	}
}

#[test]
fn single() {
	ac_tester(&[
		SetString(""),
		AssertHint("x^2"),
		Move(Movement::Up),
		AssertIndex(0),
		AssertString(""),
		AssertHint("x^2"),
		Move(Movement::Down),
		AssertIndex(0),
		AssertString(""),
		AssertHint("x^2"),
		Move(Movement::Complete),
		AssertString("x^2"),
		AssertHint(""),
		AssertIndex(0),
	]);
}

#[test]
fn multi() {
	ac_tester(&[
		SetString("s"),
		AssertHint("in("),
		Move(Movement::Up),
		AssertIndex(3),
		AssertString("s"),
		AssertHint("ignum("),
		Move(Movement::Down),
		AssertIndex(0),
		AssertString("s"),
		AssertHint("in("),
		Move(Movement::Down),
		AssertIndex(1),
		AssertString("s"),
		AssertHint("qrt("),
		Move(Movement::Up),
		AssertIndex(0),
		AssertString("s"),
		AssertHint("in("),
		Move(Movement::Complete),
		AssertString("sin("),
		AssertHint(")"),
		AssertIndex(0),
	]);
}

#[test]
fn none() {
	// string that should give no hints
	let random = "qwert987gybhj";

	ac_tester(&[
		SetString(random),
		AssertHint(""),
		Move(Movement::Up),
		AssertIndex(0),
		AssertString(random),
		AssertHint(""),
		Move(Movement::Down),
		AssertIndex(0),
		AssertString(random),
		AssertHint(""),
		Move(Movement::Complete),
		AssertString(random),
		AssertHint(""),
		AssertIndex(0),
	]);
}

#[test]
fn parens() {
	ac_tester(&[
		SetString("sin(x"),
		AssertHint(")"),
		Move(Movement::Up),
		AssertIndex(0),
		AssertString("sin(x"),
		AssertHint(")"),
		Move(Movement::Down),
		AssertIndex(0),
		AssertString("sin(x"),
		AssertHint(")"),
		Move(Movement::Complete),
		AssertString("sin(x)"),
		AssertHint(""),
		AssertIndex(0),
	]);
}
