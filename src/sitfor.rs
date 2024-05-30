/*
 * Copyright 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Horizon;
use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::pop;
use crate::situation::push;
use crate::situation::COLOR_KWD;
use crate::situation::COLOR_VAR;
use crate::situation::COLOR_NORMAL;

use crate::microparsers::identifierlen;
use crate::microparsers::is_lowercase;
use crate::microparsers::is_identifierhead;
use crate::microparsers::is_identifiertail;
use crate::microparsers::is_whitespace;
use crate::microparsers::predlen;

use crate::sitvarident::SitVarIdent;
use crate::commonargcmd::common_arg;

pub struct SitFor {}

impl Situation for SitFor {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			let len = predlen(is_lowercase, &horizon.input[i..]);
			if i + len == horizon.input.len() && (i > 0 || horizon.is_lengthenable) {
				return flush(i);
			}
			let word = &horizon.input[i..i+len];
			if word == b"in" {
				return push_forin(i);
			}
			if is_identifierhead(a) {
				return push_varident(i, 1);
			}
			if !is_whitespace(a) || a == b'\n' {
				return pop(i, 0, None);
			}
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}

pub struct SitForIn {}

impl Situation for SitForIn {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if a == b'$' {
				let candidate = &horizon.input[i+1 ..];
				let idlen = identifierlen(candidate);
				let candidate = &candidate[idlen ..];
				let spacelen = predlen(|x| x == b' ', candidate);
				let candidate = &candidate[spacelen ..];
				if let Some(end) = candidate.iter().next() {
					if idlen >= 1 && matches!(end, b';' | b'\n') {
						return become_for_in_necessarily_array(i);
					}
				} else if i > 0 || horizon.is_lengthenable {
					return flush(i);
				}
			}
			if !is_whitespace(a) || a == b'\n' {
				return become_for_in_anything_else(i);
			}
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitVarIdentNecessarilyArray {}

impl Situation for SitVarIdentNecessarilyArray {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			// An identifierhead is also an identifiertail.
			if !is_identifiertail(a) {
				return pop(i, 0, Some(b"[@]}\""));
			}
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_VAR
	}
}

pub struct SitForInAnythingElse {}

impl Situation for SitForInAnythingElse {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, _) in horizon.input.iter().enumerate() {
			if let Some(res) = common_arg(u16::from(b';'), horizon, i) {
				return res;
			}
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn push_forin(pre: usize) -> WhatNow {
	push((pre, 2, None), Box::new(SitForIn {}))
}

fn push_varident(pre: usize, len: usize) -> WhatNow {
	push((pre, len, None), Box::new(SitVarIdent { end_insert: None }))
}

fn become_for_in_necessarily_array(pre: usize) -> WhatNow {
	WhatNow {
		transform: (pre, 1, Some(b"\"${")),
		transition: Transition::Replace(Box::new(SitVarIdentNecessarilyArray {})),
	}
}

fn become_for_in_anything_else(pre: usize) -> WhatNow {
	WhatNow {
		transform: (pre, 0, None),
		transition: Transition::Replace(Box::new(SitForInAnythingElse {})),
	}
}

#[cfg(test)]
use crate::testhelpers::*;

#[test]
fn test_sit_for() {
	sit_expect!(SitFor{}, b"", &flush(0));
	sit_expect!(SitFor{}, b" ", &flush(1));
	sit_expect!(SitFor{}, b"\n", &pop(0, 0, None));
	sit_expect!(SitFor{}, b";", &pop(0, 0, None));
	sit_expect!(SitFor{}, b"_azAZ09\n", &push_varident(0, 1));
	sit_expect!(SitFor{}, b"_azAZ09;", &push_varident(0, 1));
	sit_expect!(SitFor{}, b"inn\n", &push_varident(0, 1));
	sit_expect!(SitFor{}, b"inn;", &push_varident(0, 1));
	sit_expect!(SitFor{}, b"in\n", &push_forin(0));
	sit_expect!(SitFor{}, b"in;", &push_forin(0));
	sit_expect!(SitFor{}, b"in ", &push_forin(0));
	sit_expect!(SitFor{}, b"in", &flush(0), &push_forin(0));
}

#[test]
fn test_sit_forin() {
	sit_expect!(SitForIn{}, b"", &flush(0));
	sit_expect!(SitForIn{}, b" ", &flush(1));
	sit_expect!(SitForIn{}, b"a", &become_for_in_anything_else(0));
	sit_expect!(SitForIn{}, b" a", &become_for_in_anything_else(1));
	sit_expect!(SitForIn{}, b" \n", &become_for_in_anything_else(1));
	sit_expect!(SitForIn{}, b" ;", &become_for_in_anything_else(1));
	sit_expect!(SitForIn{}, b" $a", &flush(1));
	sit_expect!(SitForIn{}, b"$a", &flush(0), &become_for_in_anything_else(0));
	sit_expect!(SitForIn{}, b" $a\n", &become_for_in_necessarily_array(1));
	sit_expect!(SitForIn{}, b" $a;", &become_for_in_necessarily_array(1));
	sit_expect!(SitForIn{}, b" $a $a;", &become_for_in_anything_else(1));
}

#[test]
fn test_sit_varidentnecessarilyarray() {
	let subj = || SitVarIdentNecessarilyArray {};

	sit_expect!(subj(), b"", &flush(0));
	sit_expect!(subj(), b"x", &flush(1));
	sit_expect!(subj(), b"x\n", &pop(1, 0, Some(b"[@]}\"")));
}

#[test]
fn test_sit_forinanythingelse() {
	let subj = || SitForInAnythingElse {};

	sit_expect!(subj(), b"", &flush(0));
	sit_expect!(subj(), b";", &pop(0, 0, None));
	sit_expect!(subj(), b"\n", &pop(0, 0, None));
}
