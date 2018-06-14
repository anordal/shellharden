/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::flush;

use ::microparsers::predlen;
use ::microparsers::is_identifierhead;
use ::microparsers::is_identifiertail;

use ::syntaxerror::UnsupportedSyntax;

use ::sitcmd::SitBeforeFirstArg;
use ::sitcmd::ArgCmdData;
use ::sitextent::SitExtent;
use ::sitstrsqesc::SitStrSqEsc;
use ::situntilbyte::SitUntilByte;
use ::sitvarident::SitVarIdent;
use ::sitvec::SitVec;

pub enum CommonStrCmdResult {
	None,
	Err(UnsupportedSyntax),
	Ok(WhatNow),
	OnlyWithQuotes(WhatNow),
	OnlyWithoutQuotes(WhatNow),
}

pub fn common_str_cmd(
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
	ctx_cmd: bool,
) -> CommonStrCmdResult {
	if horizon[i] == b'`' {
		let cmd = Box::new(SitBeforeFirstArg{
			arg_cmd_data: ArgCmdData{end_trigger: b'`' as u16, end_replace: Some(b")")},
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(cmd), pre: i, len: 1, alt: Some(b"$(")
		});
	}
	if horizon[i] == b'\\' {
		let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
		return CommonStrCmdResult::Ok(WhatNow{
			tri: Transition::Push(esc), pre: i, len: 1, alt: None
		});
	}
	if horizon[i] != b'$' {
		return CommonStrCmdResult::None;
	}
	if i+1 >= horizon.len() {
		if is_horizon_lengthenable {
			return CommonStrCmdResult::Ok(flush(i));
		}
		return CommonStrCmdResult::None;
	}
	let c = horizon[i+1];
	if c == b'\'' {
		if ctx_cmd {
			return CommonStrCmdResult::OnlyWithoutQuotes(WhatNow {
				tri: Transition::Push(Box::new(SitStrSqEsc{})),
				pre: i, len: 2, alt: None
			});
		}
	} else if c == b'(' {
		let cand: &[u8] = &horizon[i+2 ..];
		let (idlen, pos_hazard) = pos_tailhazard(cand, b')');
		if pos_hazard == cand.len() {
			if is_horizon_lengthenable {
				return CommonStrCmdResult::Ok(flush(i));
			}
		} else if idlen == 3 && pos_hazard >= 4 && cand[.. 3].eq(b"pwd") {
			let tailhazard = is_identifiertail(cand[pos_hazard]);
			let replacement: &'static [u8] = if tailhazard {
				b"${PWD}"
			} else {
				b"$PWD"
			};
			let sit = Box::new(SitExtent{
				len: 0,
				color: 0x000000ff,
				end_insert: None,
			});
			return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
				tri: Transition::Push(sit),
				pre: i, len: 6,
				alt: Some(replacement)
			});
		} else if cand.len() >= 1 && cand[0] == b'(' {
			let sit = Box::new(SitVec{
				terminator: vec!{b')', b')'},
				color: 0x00007fff,
			});
			return CommonStrCmdResult::Ok(WhatNow{
				tri: Transition::Push(sit),
				pre: i, len: 3,
				alt: None
			});
		}

		let cmd = Box::new(SitBeforeFirstArg{
			arg_cmd_data: ArgCmdData{end_trigger: b')' as u16, end_replace: None},
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(cmd),
			pre: i, len: 2, alt: None
		});
	} else if c == b'#' || c == b'?' {
		let ext = Box::new(SitExtent{
			len: 2,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::Ok(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 0, alt: None
		});
	} else if c == b'*' {
		let ext = Box::new(SitExtent{
			len: 0,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 2, alt: Some(b"$@")
		});
	} else if predlen(&|c|{c >= b'0' && c <= b'9'}, &horizon[i+1 ..]) > 1 {
		return CommonStrCmdResult::Err(UnsupportedSyntax {
			typ: "Unsupported syntax: Syntactic pitfall",
			ctx: horizon.to_owned(),
			pos: i+2,
			msg: "This does not mean what it looks like. You may be forgiven to think that the full string of \
			numerals is the variable name. Only the fist is.\n\
			\n\
			Try this and be shocked: f() { echo \"$9\" \"$10\"; }; f a b c d e f g h i j\n\
			\n\
			Here is where braces should be used to disambiguate, \
			e.g. \"${10}\" vs \"${1}0\".\n\
			\n\
			Syntactic pitfalls are deemed too dangerous to fix automatically\n\
			(the purpose of Shellharden is to fix brittle code – code that mostly \
			does what it looks like, as opposed to code that never does what it looks like):\n\
			* Fixing what it does would be 100% subtle \
			and might slip through code review unnoticed.\n\
			* Fixing its look would make a likely bug look intentional."
		});
	} else if c == b'@' || (c >= b'0' && c <= b'9') {
		let ext = Box::new(SitExtent{
			len: 2,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 0, alt: None
		});
	} else if is_identifierhead(c) {
		let tailhazard;
		if ctx_cmd {
			let cand: &[u8] = &horizon[i+1 ..];
			let (_, pos_hazard) = pos_tailhazard(cand, b'\"');
			if pos_hazard == cand.len() {
				if is_horizon_lengthenable {
					return CommonStrCmdResult::Ok(flush(i));
				}
				tailhazard = true;
			} else {
				tailhazard = is_identifiertail(cand[pos_hazard]);
			}
		} else {
			tailhazard = false;
		}
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(Box::new(SitVarIdent{
				end_insert: if_needed(tailhazard, b"}")
			})), pre: i, len: 1, alt: if_needed(tailhazard, b"${")
		});
	} else if c == b'{' {
		let cand: &[u8] = &horizon[i+2 ..];
		let (idlen, pos_hazard) = pos_tailhazard(cand, b'}');
		let mut rm_braces = false;
		let mut is_number = false;
		if pos_hazard == cand.len() {
			if is_horizon_lengthenable {
				return CommonStrCmdResult::Ok(flush(i));
			}
		} else if idlen < pos_hazard {
			rm_braces = !is_identifiertail(cand[pos_hazard]);
		} else if idlen == 0 && (cand[0] == b'#' || cand[0] == b'?') {
			is_number = true;
		}
		let wn = WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'}', color: 0x000000ff, end_replace: if_needed(rm_braces, b"")
			})), pre: i, len: 2, alt: if_needed(rm_braces, b"$")
		};
		return if is_number {
			CommonStrCmdResult::Ok(wn)
		} else {
			CommonStrCmdResult::OnlyWithQuotes(wn)
		};
	}
	return CommonStrCmdResult::Ok(flush(i+1));
}

fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	if needed { Some(val) } else { None }
}

pub fn identifierlen(horizon: &[u8]) -> usize {
	return if horizon.len() > 0 && is_identifierhead(horizon[0]) {
		1 + predlen(&is_identifiertail, &horizon[1 ..])
	} else {
		0
	}
}

fn pos_tailhazard(horizon: &[u8], end: u8) -> (usize, usize) {
	let idlen = identifierlen(&horizon);
	let mut pos = idlen;
	if idlen < horizon.len() {
		if horizon[pos] == end {
			pos += 1;
			if pos < horizon.len() {
				pos += predlen(&|x| x == b'\"', &horizon[pos ..]);
			}
		}
	}
	return (idlen, pos);
}
