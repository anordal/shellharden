
use crate::situation::WhatNow;
use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::Transition::Flush;
use crate::situation::Transition::FlushPopOnEof;
use crate::situation::Transition::Replace;
use crate::situation::Transition::Push;
use crate::situation::Transition::Pop;

pub fn whatnow_eq(a: &WhatNow, b: &WhatNow) -> bool {
	let mut eq = true;
	if a.transform.0 != b.transform.0 {
		eprintln!("WhatNow.pre: {} != {}", a.transform.0, b.transform.0);
		eq = false;
	}
	if a.transform.1 != b.transform.1 {
		eprintln!("WhatNow.len: {} != {}", a.transform.1, b.transform.1);
		eq = false;
	}
	if a.transform.2 != b.transform.2 {
		eprintln!("WhatNow.alt mismatch");
		eq = false;
	}
	transition_eq(&a.transition, &b.transition) && eq
}

fn transition_eq(a: &Transition, b: &Transition) -> bool {
	match (a, b) {
		(Flush, Flush) => true,
		(Flush, _) => {
			eprintln!("Transition mismatch; Lhs={}", "Flush");
			false
		}
		(FlushPopOnEof, FlushPopOnEof) => true,
		(FlushPopOnEof, _) => {
			eprintln!("Transition mismatch; Lhs={}", "FlushPopOnEof");
			false
		}
		(Replace(a), Replace(b)) => sit_eq(a.as_ref(), b.as_ref()),
		(Replace(_), _) => {
			eprintln!("Transition mismatch; Lhs={}", "Replace");
			false
		}
		(Push(a), Push(b)) => sit_eq(a.as_ref(), b.as_ref()),
		(Push(_), _) => {
			eprintln!("Transition mismatch; Lhs={}", "Push");
			false
		}
		(Pop, Pop) => true,
		(Pop, _) => {
			eprintln!("Transition mismatch; Lhs={}", "Pop");
			false
		}
		(Transition::Err(_), Transition::Err(_)) => true,
		(Transition::Err(_), _) => {
			eprintln!("Transition mismatch; Lhs={}", "Err");
			false
		}
	}
}

// FIXME: Compare vtable pointers.
fn sit_eq(a: &dyn Situation, b: &dyn Situation) -> bool {
	if a.get_color() != b.get_color() {
		eprintln!("Situation.color: {} != {}", a.get_color(), b.get_color());
		false
	} else {
		true
	}
}

macro_rules! sit_expect {
	($sit:expr, $horizon:expr, $expect_mid:expr, $expect_eof:expr) => {
		assert!(whatnow_eq(&$sit.whatnow($horizon, true), $expect_mid));
		assert!(whatnow_eq(&$sit.whatnow($horizon, false), $expect_eof));
	};
	($sit:expr, $horizon:expr, $expect_same:expr) => {
		assert!(whatnow_eq(&$sit.whatnow($horizon, true), $expect_same));
		assert!(whatnow_eq(&$sit.whatnow($horizon, false), $expect_same));
	};
}
