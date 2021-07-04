/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::flush_or_pop;
use crate::situation::COLOR_NORMAL;
use crate::situation::COLOR_CMD;
use crate::situation::COLOR_MAGIC;

use crate::microparsers::is_whitespace;

use crate::commonargcmd::keyword_or_command;
use crate::commonargcmd::common_arg_cmd;
use crate::commonargcmd::find_lvalue;
use crate::commonargcmd::Tri;
use crate::sitrvalue::SitRvalue;

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
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
			if is_whitespace(a) {
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

pub struct SitDeclare {
	pub end_trigger :u16,
}

impl Situation for SitDeclare {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, _) in horizon.iter().enumerate() {
			let (found, len) = find_lvalue(&horizon[i..]);
			if found == Tri::Maybe && (i > 0 || is_horizon_lengthenable) {
				return flush(i);
			}
			if found == Tri::Yes {
				return WhatNow{
					tri: Transition::Push(Box::new(SitRvalue{end_trigger: self.end_trigger})),
					pre: i + len, len: 0, alt: None
				};
			}
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_MAGIC
	}
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitextent::SitExtent;
#[cfg(test)]
use crate::sitvec::SitVec;
#[cfg(test)]
use crate::situation::COLOR_KWD;
#[cfg(test)]
use crate::situation::COLOR_HERE;

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
fn mk_kwd(len: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitExtent{len: 0, color: COLOR_KWD})),
		pre: 0, len, alt: None,
	}
}

#[test]
fn test_sit_normal() {
	let subj = || {
		SitNormal{end_trigger: 0, end_replace: None}
	};

	sit_expect!(subj(), b"fo", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"fo=", &mk_assignment(3));
	sit_expect!(subj(), b"for", &flush(0), &mk_kwd(3));
	sit_expect!(subj(), b"for=", &mk_assignment(4));
	sit_expect!(subj(), b"fork", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"fork=", &mk_assignment(5));
	sit_expect!(subj(), b";fo", &flush(1));
	sit_expect!(subj(), b";fo=", &mk_assignment(4));
	sit_expect!(subj(), b";for", &flush(1));
	sit_expect!(subj(), b";for=", &mk_assignment(5));
	sit_expect!(subj(), b";fork", &flush(1));
	sit_expect!(subj(), b";fork=", &mk_assignment(6));
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
