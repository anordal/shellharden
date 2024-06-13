/*
 * Copyright 2018 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Horizon;
use crate::situation::Situation;
use crate::situation::Transition;
use crate::sitextent::SitExtent;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::pop;
use crate::situation::push;
use crate::situation::COLOR_NORMAL;
use crate::situation::COLOR_KWD;

use crate::microparsers::predlen;
use crate::microparsers::is_lowercase;
use crate::microparsers::is_whitespace;

use crate::commonargcmd::keyword_or_command;
use crate::commonargcmd::common_expr_quoting_unneeded;

pub struct SitCase {}

impl Situation for SitCase {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, _) in horizon.input.iter().enumerate() {
			let len = predlen(is_lowercase, &horizon.input[i..]);
			if len == 0 {
				if let Some(res) = common_expr_quoting_unneeded(0x100, horizon, i) {
					return res;
				}
				continue;
			}
			if i + len == horizon.input.len() && (i > 0 || horizon.is_lengthenable) {
				return flush(i);
			}
			let word = &horizon.input[i..i+len];
			if word == b"in" {
				return become_case_in(i + len);
			}
			return flush(i + len);
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}

struct SitCaseIn {}

impl Situation for SitCaseIn {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			let len = predlen(is_lowercase, &horizon.input[i..]);
			if len == 0 {
				if a == b')' {
					return push((i, 1, None), Box::new(SitCaseArm {}));
				}
				if let Some(res) = common_expr_quoting_unneeded(0x100, horizon, i) {
					return res;
				}
				continue;
			}
			if i + len == horizon.input.len() && (i > 0 || horizon.is_lengthenable) {
				return flush(i);
			}
			let word = &horizon.input[i..i+len];
			if word == b"esac" {
				return pop_kw(i, len);
			}
			return flush(i + len);
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitCaseArm {}

impl Situation for SitCaseArm {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if a == b';' {
				if i + 1 < horizon.input.len() {
					if horizon.input[i + 1] == b';' {
						return pop(i, 0, None);
					}
				} else if i > 0 || horizon.is_lengthenable {
					return flush(i);
				}
			}
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			// Premature esac: Survive and rewrite.
			let len = predlen(is_lowercase, &horizon.input[i..]);
			if i + len != horizon.input.len() || (i == 0 && !horizon.is_lengthenable) {
				let word = &horizon.input[i..i+len];
				if word == b"esac" {
					return pop(i, 0, Some(b";; "));
				}
			}
			return keyword_or_command(0x100, horizon, i);
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn become_case_in(pre: usize) -> WhatNow {
	WhatNow{
		transform: (pre, 0, None),
		transition: Transition::Replace(Box::new(SitCaseIn {})),
	}
}

fn pop_kw(pre: usize, len: usize) -> WhatNow {
	WhatNow {
		transform: (pre, len, None),
		transition: Transition::Replace(Box::new(SitExtent { len: 0, color: COLOR_KWD })),
	}
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitcmd::SitCmd;
#[cfg(test)]
use crate::situation::COLOR_ESC;
#[cfg(test)]
use crate::sitextent::push_extent;

#[test]
fn test_sit_case() {
	sit_expect!(SitCase{}, b"", &flush(0));
	sit_expect!(SitCase{}, b" ", &flush(1));
	sit_expect!(SitCase{}, b"i\"", &flush(1));
	sit_expect!(SitCase{}, b"i", &flush(0), &flush(1));
	sit_expect!(SitCase{}, b"in ", &become_case_in(2));
	sit_expect!(SitCase{}, b"in", &flush(0), &become_case_in(2));
	sit_expect!(SitCase{}, b"inn", &flush(0), &flush(3));
	sit_expect!(SitCase{}, b" in", &flush(1));
	sit_expect!(SitCase{}, b"fin", &flush(0), &flush(3));
	sit_expect!(SitCase{}, b"fin ", &flush(3));
}

#[test]
fn test_sit_casein() {
	sit_expect!(SitCaseIn{}, b"", &flush(0));
	sit_expect!(SitCaseIn{}, b" ", &flush(1));
	sit_expect!(SitCaseIn{}, b"esa\"", &flush(3));
	sit_expect!(SitCaseIn{}, b"esa", &flush(0), &flush(3));
	sit_expect!(SitCaseIn{}, b"esac ", &pop_kw(0, 4));
	sit_expect!(SitCaseIn{}, b"esac", &flush(0), &pop_kw(0, 4));
	sit_expect!(SitCaseIn{}, b"esacs", &flush(0), &flush(5));
	sit_expect!(SitCaseIn{}, b" esac", &flush(1));
	sit_expect!(SitCaseIn{}, b"besac", &flush(0), &flush(5));
	sit_expect!(SitCaseIn{}, b"besac ", &flush(5));
}

#[test]
fn test_sit_casearm() {
	let found_command = push((0, 0, None), Box::new(SitCmd{end_trigger: 0x100}));
	let found_the_esac_word = pop(0, 0, Some(b";; "));

	sit_expect!(SitCaseArm{}, b"", &flush(0));
	sit_expect!(SitCaseArm{}, b" ", &flush(1));
	sit_expect!(SitCaseArm{}, b"\\", &push_extent(COLOR_ESC, 0, 2));
	sit_expect!(SitCaseArm{}, b";", &flush(0), &flush(1));
	sit_expect!(SitCaseArm{}, b"; ", &flush(2));
	sit_expect!(SitCaseArm{}, b" ;", &flush(1));
	sit_expect!(SitCaseArm{}, b"esa", &flush(0), &found_command);
	sit_expect!(SitCaseArm{}, b"esac ", &found_the_esac_word);
	sit_expect!(SitCaseArm{}, b"esac", &flush(0), &found_the_esac_word);
	sit_expect!(SitCaseArm{}, b"esacs", &flush(0), &found_command);
	sit_expect!(SitCaseArm{}, b" esac", &flush(1));
	sit_expect!(SitCaseArm{}, b"besac", &flush(0), &found_command);
	sit_expect!(SitCaseArm{}, b"besac ", &found_command);
}
