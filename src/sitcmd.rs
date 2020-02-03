/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Situation;
use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::flush;
use ::situation::flush_or_pop;
use ::situation::COLOR_NORMAL;
use ::situation::COLOR_CMD;

use ::microparsers::is_whitespace;

use ::commonargcmd::keyword_or_command;
use ::commonargcmd::common_arg_cmd;

pub struct SitNormal {
	pub end_trigger :u16,
	pub end_replace :Option<&'static [u8]>,
}

impl Situation for SitNormal {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			if u16::from(a) == self.end_trigger {
				return WhatNow{
					tri: Transition::Pop, pre: i, len: 1,
					alt: self.end_replace
				};
			}
			return keyword_or_command(
				self.end_trigger, &horizon, i, is_horizon_lengthenable
			);
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

pub struct SitCmd {
	pub end_trigger :u16,
}

impl Situation for SitCmd {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b' ' || a == b'\t' {
				return WhatNow{
					tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
					pre: i, len: 1, alt: None
				};
			}
			if a == b'(' {
				return WhatNow{
					tri: Transition::Pop, pre: i, len: 0, alt: None
				};
			}
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

struct SitArg {
	end_trigger :u16,
}

impl Situation for SitArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, _) in horizon.iter().enumerate() {
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

#[cfg(test)]
use ::testhelpers::*;
#[cfg(test)]
use sitrvalue::SitRvalue;
#[cfg(test)]
use sitextent::SitExtent;
#[cfg(test)]
use sitvec::SitVec;
#[cfg(test)]
use situation::COLOR_KWD;
#[cfg(test)]
use situation::COLOR_HERE;

#[cfg(test)]
fn mk_assignment(pre: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitRvalue{end_trigger: 0})),
		pre, len: 0, alt: None
	}
}

#[cfg(test)]
fn mk_cmd(pre: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitCmd{end_trigger: 0})),
		pre, len: 0, alt: None
	}
}

#[cfg(test)]
fn mk_kwd(pre: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitExtent{
			len: 0,
			color: COLOR_KWD,
			end_insert: None
		})), pre, len: 0, alt: None
	}
}

#[test]
fn test_sit_normal() {
	let subj = || {
		SitNormal{end_trigger: 0, end_replace: None}
	};

	sit_expect!(subj(), b"esa", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"esa=", &mk_assignment(4));
	sit_expect!(subj(), b"esac", &flush(0), &mk_kwd(0));
	sit_expect!(subj(), b"esac=", &mk_assignment(5));
	sit_expect!(subj(), b"esack", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"esack=", &mk_assignment(6));
	sit_expect!(subj(), b";esa", &flush(1));
	sit_expect!(subj(), b";esa=", &mk_assignment(5));
	sit_expect!(subj(), b";esac", &flush(1));
	sit_expect!(subj(), b";esac=", &mk_assignment(6));
	sit_expect!(subj(), b";esack", &flush(1));
	sit_expect!(subj(), b";esack=", &mk_assignment(7));
}

#[test]
fn test_sit_arg() {
	let found_heredoc = WhatNow{
		tri: Transition::Push(Box::new(
			SitVec{terminator: vec![b'\\'], color: COLOR_HERE}
		)),
		pre: 0, len: 8, alt: None
	};
	let subj = || {
		SitArg{end_trigger: 0}
	};

	sit_expect!(subj(), b"", &flush_or_pop(0));
	sit_expect!(subj(), b" ", &flush_or_pop(1));
	sit_expect!(subj(), b"arg", &flush_or_pop(3));
	sit_expect!(subj(), b"<<- \"\\\\\"\n", &found_heredoc);
	sit_expect!(subj(), b"a <<- \"\\\\\"", &flush(2));
	sit_expect!(subj(), b"a <<- \"\\", &flush(2));
	sit_expect!(subj(), b"a <<- ", &flush(2));
	sit_expect!(subj(), b"a <", &flush(2));
	sit_expect!(subj(), b"a ", &flush_or_pop(2));
}
