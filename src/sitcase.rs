/*
 * Copyright 2018 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::Transition;
use crate::sitextent::SitExtent;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::COLOR_NORMAL;
use crate::situation::COLOR_KWD;

use crate::microparsers::predlen;
use crate::microparsers::is_whitespace;
use crate::microparsers::is_word;

use crate::commonargcmd::keyword_or_command;
use crate::commonargcmd::common_no_cmd_quoting_unneeded;

pub struct SitIn {}

impl Situation for SitIn {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, _) in horizon.iter().enumerate() {
			let len = predlen(is_word, &horizon[i..]);
			if len == 0 {
				continue;
			}
			if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
				return flush(i);
			}
			let word = &horizon[i..i+len];
			if word == b"in" {
				return WhatNow{
					tri: Transition::Replace(Box::new(SitCase{})),
					pre: i + len, len: 0, alt: None
				};
			}
			if let Some(res) = common_no_cmd_quoting_unneeded(
				0x100, horizon, i, is_horizon_lengthenable
			) {
				return res;
			}
			return flush(i + len);
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}

struct SitCase {}

impl Situation for SitCase {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			let len = predlen(is_word, &horizon[i..]);
			if len == 0 {
				if a == b')' {
					return WhatNow{
						tri: Transition::Push(Box::new(SitCaseArm{})),
						pre: i, len: 1, alt: None
					};
				}
				continue;
			}
			if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
				return flush(i);
			}
			let word = &horizon[i..i+len];
			if word == b"esac" {
				return pop_kw(i, len);
			}
			if let Some(res) = common_no_cmd_quoting_unneeded(
				0x100, horizon, i, is_horizon_lengthenable
			) {
				return res;
			}
			return flush(i + len);
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitCaseArm {}

impl Situation for SitCaseArm {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b';' {
				if i + 1 < horizon.len() {
					if horizon[i + 1] == b';' {
						return WhatNow{
							tri: Transition::Pop, pre: i, len: 0, alt: None
						};
					}
				} else if i > 0 || is_horizon_lengthenable {
					return flush(i);
				}
			}
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			// Premature esac: Survive and rewrite.
			let len = predlen(is_word, &horizon[i..]);
			if i + len != horizon.len() || (i == 0 && !is_horizon_lengthenable) {
				let word = &horizon[i..i+len];
				if word == b"esac" {
					return WhatNow{
						tri: Transition::Pop, pre: i, len: 0, alt: Some(b";; ")
					};
				}
			}
			return keyword_or_command(0x100, &horizon, i, is_horizon_lengthenable);
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn pop_kw(pre: usize, len: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Replace(Box::new(SitExtent{
			len: 0,
			color: COLOR_KWD,
			end_insert: None
		})), pre, len, alt: None
	}
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitcmd::SitCmd;

#[test]
fn test_sit_in() {
	sit_expect!(SitIn{}, b"", &flush(0));
	sit_expect!(SitIn{}, b" ", &flush(1));
	sit_expect!(SitIn{}, b"i", &flush(0), &flush(1));
	let found_the_in_word = WhatNow{
		tri: Transition::Replace(Box::new(SitCase{})),
		pre: 2, len: 0, alt: None
	};
	sit_expect!(SitIn{}, b"in ", &found_the_in_word);
	sit_expect!(SitIn{}, b"in", &flush(0), &found_the_in_word);
	sit_expect!(SitIn{}, b"inn", &flush(0), &flush(3));
	sit_expect!(SitIn{}, b" in", &flush(1));
	sit_expect!(SitIn{}, b"fin", &flush(0), &flush(3));
	sit_expect!(SitIn{}, b"fin ", &flush(3));
}

#[test]
fn test_sit_case() {
	sit_expect!(SitCase{}, b"", &flush(0));
	sit_expect!(SitCase{}, b" ", &flush(1));
	sit_expect!(SitCase{}, b"esa", &flush(0), &flush(3));
	sit_expect!(SitCase{}, b"esac ", &pop_kw(0, 4));
	sit_expect!(SitCase{}, b"esac", &flush(0), &pop_kw(0, 4));
	sit_expect!(SitCase{}, b"esacs", &flush(0), &flush(5));
	sit_expect!(SitCase{}, b" esac", &flush(1));
	sit_expect!(SitCase{}, b"besac", &flush(0), &flush(5));
	sit_expect!(SitCase{}, b"besac ", &flush(5));
}

#[test]
fn test_sit_casearm() {
	sit_expect!(SitCaseArm{}, b"", &flush(0));
	sit_expect!(SitCaseArm{}, b" ", &flush(1));
	let found_command = WhatNow{
		tri: Transition::Push(Box::new(SitCmd{end_trigger: 0x100})),
		pre: 0, len: 0, alt: None
	};
	sit_expect!(SitCaseArm{}, b"esa", &flush(0), &found_command);
	let found_the_esac_word = WhatNow{
		tri: Transition::Pop,
		pre: 0, len: 0, alt: Some(b";; ")
	};
	sit_expect!(SitCaseArm{}, b"esac ", &found_the_esac_word);
	sit_expect!(SitCaseArm{}, b"esac", &flush(0), &found_the_esac_word);
	sit_expect!(SitCaseArm{}, b"esacs", &flush(0), &found_command);
	sit_expect!(SitCaseArm{}, b" esac", &flush(1));
	sit_expect!(SitCaseArm{}, b"besac", &flush(0), &found_command);
	sit_expect!(SitCaseArm{}, b"besac ", &found_command);
}
