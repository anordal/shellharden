/*
 * Copyright 2016 - 2019 Andreas Nordal
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
use crate::situation::flush_or_pop;
use crate::situation::pop;
use crate::situation::COLOR_NORMAL;
use crate::situation::COLOR_CMD;
use crate::situation::COLOR_ESC;

use crate::microparsers::is_whitespace;

use crate::commonargcmd::keyword_or_command;
use crate::commonargcmd::common_arg;
use crate::commonargcmd::common_cmd;

use crate::sitextent::push_extent;

pub struct SitNormal {
	pub end_trigger :u16,
	pub end_replace :Option<&'static [u8]>,
}

impl Situation for SitNormal {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			if a == b'\\' {
				return push_extent(COLOR_ESC, i, 2);
			}
			if u16::from(a) == self.end_trigger {
				return pop(i, 1, self.end_replace);
			}
			return keyword_or_command(self.end_trigger, horizon, i);
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

pub struct SitCmd {
	pub end_trigger :u16,
}

impl Situation for SitCmd {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if let Some(res) = common_cmd(self.end_trigger, horizon, i) {
				return res;
			}
			if is_whitespace(a) {
				return WhatNow {
					transform: (i, 1, None),
					transition: Transition::Replace(Box::new(SitArg {
						end_trigger: self.end_trigger,
					})),
				};
			}
			if a == b'(' {
				return pop(i, 0, None);
			}
		}
		flush_or_pop(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

pub struct SitArg {
	pub end_trigger :u16,
}

impl Situation for SitArg {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, _) in horizon.input.iter().enumerate() {
			if let Some(res) = common_arg(self.end_trigger, horizon, i) {
				return res;
			}
		}
		flush_or_pop(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitmagic::push_magic;
#[cfg(test)]
use crate::sitrvalue::SitRvalue;
#[cfg(test)]
use crate::sitvec::SitVec;
#[cfg(test)]
use crate::sitfor::SitFor;
#[cfg(test)]
use crate::situation::COLOR_HERE;
#[cfg(test)]
use crate::situation::push;

#[cfg(test)]
fn mk_assignment(pre: usize) -> WhatNow {
	push((pre, 0, None), Box::new(SitRvalue { end_trigger: 0 }))
}

#[cfg(test)]
fn mk_cmd(pre: usize) -> WhatNow {
	push((pre, 0, None), Box::new(SitCmd { end_trigger: 0 }))
}

#[test]
fn test_sit_normal() {
	let subj = || {
		SitNormal{end_trigger: 0, end_replace: None}
	};

	sit_expect!(subj(), b"\\", &push_extent(COLOR_ESC, 0, 2));
	sit_expect!(subj(), b"fo", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"fo=", &mk_assignment(3));
	sit_expect!(subj(), b"for", &flush(0), &push((0, 3, None), Box::new(SitFor {})));
	sit_expect!(subj(), b"for=", &mk_assignment(4));
	sit_expect!(subj(), b"fork", &flush(0), &mk_cmd(0));
	sit_expect!(subj(), b"fork=", &mk_assignment(5));
	sit_expect!(subj(), b";fo", &flush(1));
	sit_expect!(subj(), b";fo=", &mk_assignment(4));
	sit_expect!(subj(), b";for", &flush(1));
	sit_expect!(subj(), b";for=", &mk_assignment(5));
	sit_expect!(subj(), b";fork", &flush(1));
	sit_expect!(subj(), b";fork=", &mk_assignment(6));
	sit_expect!(subj(), b"((", &flush(0), &push_magic(0, 1, b')'));
	sit_expect!(subj(), b"[[", &flush(0), &push_magic(0, 1, b']'));
}

#[test]
fn test_sit_arg() {
	let found_heredoc = push(
		(0, 8, None),
		Box::new(SitVec {
			terminator: vec![b'\\'],
			color: COLOR_HERE,
		}),
	);
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
