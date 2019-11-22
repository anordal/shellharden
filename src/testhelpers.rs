
use ::situation::WhatNow;
use ::situation::Situation;
use ::situation::Transition;
use ::situation::Transition::Flush;
use ::situation::Transition::FlushPopOnEof;
use ::situation::Transition::Replace;
use ::situation::Transition::Push;
use ::situation::Transition::Pop;

pub fn whatnow_eq(a: &WhatNow, b: &WhatNow) -> bool {
	if a.pre != b.pre {
		eprintln!("WhatNow.pre: {} != {}", a.pre, b.pre);
		false
	} else if a.len != b.len {
		eprintln!("WhatNow.len: {} != {}", a.len, b.len);
		false
	} else if a.alt != b.alt {
		eprintln!("WhatNow.alt mismatch");
		false
	} else {
		transition_eq(&a.tri, &b.tri)
	}
}

fn transition_eq(a: &Transition, b: &Transition) -> bool {
	match (a, b) {
		(Flush, Flush) => true,
		(FlushPopOnEof, FlushPopOnEof) => true,
		(Replace(a), Replace(b)) => {
			sit_eq(a.as_ref(), b.as_ref())
		},
		(Push(a), Push(b)) => {
			sit_eq(a.as_ref(), b.as_ref())
		},
		(Pop, Pop) => true,
		_ => {
			eprintln!("Transition mismatch");
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
