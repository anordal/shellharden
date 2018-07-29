/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::ParseResult;
use ::situation::flush;
use ::situation::COLOR_BOLD;

use ::microparsers::prefixlen;
use ::microparsers::predlen;
use ::microparsers::identifierlen;
use ::microparsers::is_whitespace;
use ::microparsers::is_word;

use ::commonstrcmd::CommonStrCmdResult;
use ::commonstrcmd::common_str_cmd;

use ::sitcase::SitIn;
use ::sitcmd::SitNormal;
use ::sitcmd::SitCmd;
use ::sitcomment::SitComment;
use ::sitextent::SitExtent;
use ::sitrvalue::SitRvalue;
use ::sitstrdq::SitStrDq;
use ::sitstrphantom::SitStrPhantom;
use ::situntilbyte::SitUntilByte;
use ::sitvec::SitVec;

pub fn keyword_or_command(
	end_trigger :u16,
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
) -> WhatNow {
	if horizon[i] == b'(' {
		return WhatNow{
			tri: Transition::Push(Box::new(SitNormal{
				end_trigger: b')' as u16, end_replace: None
			})), pre: i, len: 1, alt: None
		};
	}
	let mut len = identifierlen(&horizon[i..]);
	if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
		return flush(i);
	}
	if len > 0 && i + len < horizon.len() {
		if horizon[i + len] == b'+' && i + len + 1 < horizon.len() {
			len += 1;
		}
		if horizon[i + len] == b'=' {
			return WhatNow{
				tri: Transition::Push(Box::new(SitRvalue{end_trigger: end_trigger})),
				pre: i + len + 1, len: 0, alt: None
			};
		}
	}
	let len = len + predlen(&is_word, &horizon[i+len..]);
	if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
		return flush(i);
	}
	let word = &horizon[i..i+len];
	match word {
		b"[[" => WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: vec!{b']', b']'}, color: 0x00007fff}
			)),
			pre: i, len: len, alt: None
		},
		b"case" => WhatNow{
			tri: Transition::Push(Box::new(SitIn{end_trigger: end_trigger})),
			pre: i, len: len, alt: None
		},
		b"do" |
		b"done" |
		b"elif" |
		b"else" |
		b"esac" |
		b"fi" |
		b"for" |
		b"if" |
		b"select" |
		b"then" |
		b"until" |
		b"while" |
		b"{" |
		b"}" => WhatNow{
			tri: Transition::Push(Box::new(SitExtent{
				len: len,
				color: COLOR_BOLD | 0x800080,
				end_insert: None
			})), pre: i, len: 0, alt: None
		},
		_ => WhatNow{
			tri: Transition::Push(Box::new(SitCmd{end_trigger: end_trigger})),
			pre: i, len: 0, alt: None
		},
	}
}

// Does not pop on eof → Callers must use flush_or_pop
pub fn common_arg_cmd(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<ParseResult> {
	let plen = prefixlen(&horizon[i..], b">&");
	if plen == 2 {
		return Some(Ok(flush(i + 2)));
	}
	if i + plen == horizon.len() && (i > 0 || is_horizon_lengthenable) {
		return Some(Ok(flush(i)));
	}
	let a = horizon[i];
	if a == b'\n' || a == b';' || a == b'|' || a == b'&' {
		return Some(Ok(WhatNow{
			tri: Transition::Pop, pre: i, len: 0, alt: None
		}));
	}
	common_arg_cmd_array(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_arg_cmd_array(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<ParseResult> {
	let a = horizon[i];
	if a as u16 == end_trigger {
		return Some(Ok(WhatNow{
			tri: Transition::Pop, pre: i, len: 0, alt: None
		}));
	}
	if a == b'#' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitComment{})),
			pre: i, len: 1, alt: None
		}));
	}
	if a == b'\'' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'\'', color: 0x00ffff00, end_replace: None
			})),
			pre: i, len: 1, alt: None
		}));
	}
	if a == b'\"' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitStrDq{})),
			pre: i, len: 1, alt: None
		}));
	}
	match common_str_cmd(&horizon, i, is_horizon_lengthenable, true) {
		CommonStrCmdResult::None => {},
		CommonStrCmdResult::Err(e) => { return Some(Err(e)); },
		CommonStrCmdResult::Ok(consult)
		| CommonStrCmdResult::OnlyWithoutQuotes(consult)=> {
			return Some(Ok(consult));
		},
		CommonStrCmdResult::OnlyWithQuotes(_) => {
			return Some(Ok(WhatNow{
				tri: Transition::Push(Box::new(SitStrPhantom{
					cmd_end_trigger: end_trigger,
				})), pre: i, len: 0, alt: Some(b"\"")
			}));
		},
	}
	let (ate, delimiter) = find_heredoc(&horizon[i ..]);
	if i + ate == horizon.len() {
		if i > 0 || is_horizon_lengthenable {
			return Some(Ok(flush(i)));
		}
	} else if delimiter.len() > 0 {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: delimiter, color: 0x0077ff00}
			)),
			pre: i, len: ate, alt: None
		}));
	} else if ate > 0 {
		return Some(Ok(flush(i + ate)));
	}
	None
}

fn find_heredoc(horizon: &[u8]) -> (usize, Vec<u8>) {
	let mut ate = predlen(&|x| x == b'<', &horizon);
	let mut found = Vec::<u8>::new();
	if ate != 2 {
		return (ate, found);
	}
	ate += predlen(&|x| x == b'-', &horizon[ate ..]);
	ate += predlen(&is_whitespace, &horizon[ate ..]);

	// Lex one word.
	let herein = &horizon[ate ..];
	found.reserve(herein.len());

	#[derive(Clone)]
	#[derive(Copy)]
	enum DelimiterSyntax {
		WORD,
		WORDESC,
		SQ,
		DQ,
		DQESC,
	}
	let mut state = DelimiterSyntax::WORD;

	for byte_ref in herein {
		let byte: u8 = *byte_ref;
		state = match (state, byte) {
			(DelimiterSyntax::WORD, b' ' ) => break,
			(DelimiterSyntax::WORD, b'\n') => break,
			(DelimiterSyntax::WORD, b'\t') => break,
			(DelimiterSyntax::WORD, b'\\') => DelimiterSyntax::WORDESC,
			(DelimiterSyntax::WORD, b'\'') => DelimiterSyntax::SQ,
			(DelimiterSyntax::WORD, b'\"') => DelimiterSyntax::DQ,
			(DelimiterSyntax::SQ, b'\'') => DelimiterSyntax::WORD,
			(DelimiterSyntax::DQ, b'\"') => DelimiterSyntax::WORD,
			(DelimiterSyntax::DQ, b'\\') => DelimiterSyntax::DQESC,
			(DelimiterSyntax::WORDESC, b'\n') => DelimiterSyntax::WORD,
			(DelimiterSyntax::WORDESC, _) => {
				found.push(byte);
				DelimiterSyntax::WORD
			},
			(DelimiterSyntax::DQESC, b'\n') => DelimiterSyntax::DQ,
			(DelimiterSyntax::DQESC, _) => {
				if byte != b'\"' && byte != b'\\' {
					found.push(b'\\');
				}
				found.push(byte);
				DelimiterSyntax::DQ
			},
			(_, _) => {
				found.push(byte);
				state
			},
		};
		ate += 1;
	}
	return (ate, found);
}
