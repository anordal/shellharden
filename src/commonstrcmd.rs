/*
 * Copyright 2016 - 2022 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::push;
use crate::situation::if_needed;
use crate::situation::COLOR_ESC;
use crate::situation::COLOR_VAR;

use crate::microparsers::predlen;
use crate::microparsers::is_identifierhead;
use crate::microparsers::is_identifiertail;
use crate::microparsers::identifierlen;

use crate::syntaxerror::UnsupportedSyntax;

use crate::sitcmd::SitNormal;
use crate::sitextent::push_extent;
use crate::sitmagic::push_magic;
use crate::sitvarbrace::SitVarBrace;
use crate::sitvarident::SitVarIdent;

#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum QuotingCtx {
	Need,
	Dontneed,
	Interpolation,
}

pub enum CommonStrCmdResult {
	None,
	Some(WhatNow),
	OnlyWithQuotes(WhatNow),
}

pub fn common_str_cmd(
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
	ctx: QuotingCtx,
) -> CommonStrCmdResult {
	let need_quotes = ctx == QuotingCtx::Need;
	let is_interpolation = ctx == QuotingCtx::Interpolation;

	if horizon[i] == b'`' {
		let found_pwd = find_pwd(horizon, i, 1, b'`', is_horizon_lengthenable);
		match found_pwd {
			CommonStrCmdResult::None => {}
			CommonStrCmdResult::Some(_) |
			CommonStrCmdResult::OnlyWithQuotes(_) => {
				return found_pwd;
			}
		}
		return CommonStrCmdResult::OnlyWithQuotes(push(
			(i, 1, Some(b"$(")),
			Box::new(SitNormal {
				end_trigger: u16::from(b'`'),
				end_replace: Some(b")"),
			}),
		));
	}
	if horizon[i] == b'\\' {
		return CommonStrCmdResult::Some(push_extent(COLOR_ESC, i, 2, None));
	}
	if horizon[i] != b'$' {
		return CommonStrCmdResult::None;
	}
	if i+1 >= horizon.len() {
		if i > 0 || is_horizon_lengthenable {
			return CommonStrCmdResult::Some(flush(i));
		}
		return CommonStrCmdResult::None;
	}
	let c = horizon[i+1];
	if c == b'(' {
		let found_pwd = find_pwd(horizon, i, 2, b')', is_horizon_lengthenable);
		match found_pwd {
			CommonStrCmdResult::None => {}
			CommonStrCmdResult::Some(_) |
			CommonStrCmdResult::OnlyWithQuotes(_) => {
				return found_pwd;
			}
		}
		if i+2 >= horizon.len() {
			// Reachable, but already handled by find_pwd.
		} else if horizon[i+2] == b'(' {
			return CommonStrCmdResult::Some(push_magic(i, 2, b')'));
		}
		return CommonStrCmdResult::OnlyWithQuotes(push(
			(i, 2, None),
			Box::new(SitNormal {
				end_trigger: u16::from(b')'),
				end_replace: None,
			}),
		));
	} else if is_variable_of_numeric_content(c) {
		return CommonStrCmdResult::Some(push_extent(COLOR_VAR, i, 2, None));
	} else if c == b'@' || c == b'*' || c == b'-' || is_decimal(c) {
		if predlen(is_decimal, &horizon[i+1 ..]) > 1 {
			return bail_doubledigit(horizon, i+2);
		}
		return CommonStrCmdResult::OnlyWithQuotes(push_extent(COLOR_VAR, i, 2, None));
	} else if is_identifierhead(c) {
		let tailhazard;
		if need_quotes {
			let cand: &[u8] = &horizon[i+1 ..];
			let (_, pos_hazard) = pos_tailhazard(cand, b'\"');
			if pos_hazard == cand.len() {
				if i > 0 || is_horizon_lengthenable {
					return CommonStrCmdResult::Some(flush(i));
				}
				tailhazard = true;
			} else {
				tailhazard = is_identifiertail(cand[pos_hazard]);
			}
		} else {
			tailhazard = false;
		}
		return CommonStrCmdResult::OnlyWithQuotes(push(
			(i, 1, if_needed(tailhazard, b"${")),
			Box::new(SitVarIdent {
				end_insert: if_needed(tailhazard, b"}"),
			}),
		));
	} else if c == b'{' {
		let cand: &[u8] = &horizon[i+2 ..];
		let (idlen, pos_hazard) = pos_tailhazard(cand, b'}');
		let mut rm_braces = false;
		let mut is_number = false;
		if pos_hazard == cand.len() {
			if i > 0 || is_horizon_lengthenable {
				return CommonStrCmdResult::Some(flush(i));
			}
		} else if idlen == 0 {
			is_number = is_variable_of_numeric_content(cand[0]);
		} else if idlen < pos_hazard && !is_identifiertail(cand[pos_hazard]) {
			let is_interpolation = is_interpolation || pos_hazard - idlen == 1;
			rm_braces = need_quotes || !is_interpolation;
		}
		let wn = push(
			(i, 2, if_needed(rm_braces, b"$")),
			Box::new(SitVarBrace::new(rm_braces, need_quotes)),
		);
		return if is_number {
			CommonStrCmdResult::Some(wn)
		} else {
			CommonStrCmdResult::OnlyWithQuotes(wn)
		};
	}
	CommonStrCmdResult::None
}

fn find_pwd(
	horizon: &[u8],
	i: usize,
	candidate_offset: usize,
	end: u8,
	is_horizon_lengthenable: bool,
) -> CommonStrCmdResult {
	let cand: &[u8] = &horizon[i + candidate_offset ..];
	let (idlen, pos_hazard) = pos_tailhazard(cand, end);
	if pos_hazard == cand.len() {
		if i > 0 || is_horizon_lengthenable {
			return CommonStrCmdResult::Some(flush(i));
		}
	} else if idlen == 3 && pos_hazard >= 4 && cand[.. 3].eq(b"pwd") {
		let tailhazard = is_identifiertail(cand[pos_hazard]);
		let replacement: &'static [u8] = if tailhazard {
			b"${PWD}"
		} else {
			b"$PWD"
		};
		let what = push_extent(COLOR_VAR, i, candidate_offset + idlen + 1, Some(replacement));
		return CommonStrCmdResult::OnlyWithQuotes(what);
	}
	CommonStrCmdResult::None
}

fn pos_tailhazard(horizon: &[u8], end: u8) -> (usize, usize) {
	let idlen = identifierlen(horizon);
	let mut pos = idlen;
	if pos < horizon.len() && horizon[pos] == end {
		pos += 1;
		pos += predlen(|x| x == b'\"', &horizon[pos ..]);
	}
	(idlen, pos)
}

fn is_decimal(byte: u8) -> bool {
	matches!(byte, b'0' ..= b'9')
}

fn is_variable_of_numeric_content(c: u8) -> bool {
	matches!(c, b'#' | b'?' | b'$' | b'!')
}

fn bail_doubledigit(context: &[u8], pos: usize) -> CommonStrCmdResult {
	CommonStrCmdResult::Some(WhatNow {
		transform: (0, 0, None),
		transition: Transition::Err(UnsupportedSyntax {
			typ: "Unsupported syntax: Syntactic pitfall",
			ctx: context.to_owned(),
			pos,
			msg: "This does not mean what it looks like. You may be forgiven to think that the full string of \
			numerals is the variable name. Only the fist is.\n\
			\n\
			Try this and be shocked: f() { echo \"$9\" \"$10\"; }; f a b c d e f g h i j\n\
			\n\
			Here is where braces should be used to disambiguate, \
			e.g. \"${10}\" vs \"${1}0\".\n\
			\n\
			Syntactic pitfalls are deemed too dangerous to fix automatically\n\
			(the purpose of Shellharden is to fix vulnerable code – code that mostly \
			does what it looks like, as opposed to code that never does what it looks like):\n\
			* Fixing what it does would be 100% subtle \
			and might slip through code review unnoticed.\n\
			* Fixing its look would make a likely bug look intentional."
		}),
	})
}
