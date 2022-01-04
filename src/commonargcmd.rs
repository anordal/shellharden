/*
 * Copyright 2016 - 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::if_needed;
use crate::situation::COLOR_HERE;
use crate::situation::COLOR_KWD;
use crate::situation::COLOR_SQ;
use crate::situation::COLOR_VAR;

use crate::microparsers::prefixlen;
use crate::microparsers::predlen;
use crate::microparsers::identifierlen;
use crate::microparsers::is_whitespace;
use crate::microparsers::is_word;

use crate::commonstrcmd::CommonStrCmdResult;
use crate::commonstrcmd::common_str_cmd;

use crate::sitcase::SitCase;
use crate::sitfor::SitFor;
use crate::sitcmd::SitNormal;
use crate::sitcmd::SitCmd;
use crate::sitcomment::SitComment;
use crate::sitextent::SitExtent;
use crate::sitmagic::push_magic;
use crate::sitrvalue::SitRvalue;
use crate::sitstrdq::SitStrDq;
use crate::sitstrphantom::SitStrPhantom;
use crate::sitstrsqesc::SitStrSqEsc;
use crate::sittest::SitTest;
use crate::situntilbyte::SitUntilByte;
use crate::sitvec::SitVec;

pub fn keyword_or_command(
	end_trigger :u16,
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
) -> WhatNow {
	if horizon[i] == b'#' {
		return push_comment(i);
	}
	let (found, len) = find_lvalue(&horizon[i..]);
	if found == Tri::Maybe && (i > 0 || is_horizon_lengthenable) {
		return flush(i);
	}
	if found == Tri::Yes {
		return WhatNow{
			tri: Transition::Push(Box::new(SitRvalue{end_trigger})),
			pre: i + len, len: 0, alt: None
		};
	}
	let len = predlen(is_word, &horizon[i..]);
	let len = if len != 0 { len } else { prefixlen(&horizon[i..], b"((") };
	if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
		return flush(i);
	}
	let word = &horizon[i..i+len];
	match word {
		b"(" => WhatNow{
			tri: Transition::Push(Box::new(SitNormal{
				end_trigger: u16::from(b')'), end_replace: None
			})), pre: i, len: 1, alt: None
		},
		b"((" => push_magic(i, 1, b')'),
		b"[[" => push_magic(i, 1, b']'),
		b"case" => WhatNow{
			tri: Transition::Push(Box::new(SitCase{})),
			pre: i, len, alt: None
		},
		b"for" |
		b"select" => WhatNow{
			tri: Transition::Push(Box::new(SitFor{})),
			pre: i, len, alt: None
		},
		b"!" |
		b"declare" |
		b"do" |
		b"done" |
		b"elif" |
		b"else" |
		b"export" |
		b"fi" |
		b"function" |
		b"if" |
		b"local" |
		b"readonly" |
		b"then" |
		b"until" |
		b"while" |
		b"{" |
		b"}" => WhatNow{
			tri: Transition::Push(Box::new(SitExtent{len: 0, color: COLOR_KWD})),
			pre: i, len, alt: None,
		},
		b"[" |
		b"test" => WhatNow{
			tri: Transition::Push(Box::new(SitTest{end_trigger})),
			pre: i, len, alt: None,
		},
		_ => WhatNow{
			tri: Transition::Push(Box::new(SitCmd{end_trigger})),
			pre: i, len: 0, alt: None
		},
	}
}

pub fn common_arg(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i, is_horizon_lengthenable) {
		return Some(res);
	}
	common_expr(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_cmd(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i, is_horizon_lengthenable) {
		return Some(res);
	}
	common_token(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_expr(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if horizon[i] == b'#' {
		return Some(push_comment(i));
	}
	common_token(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_token(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if let Some(res) = find_usual_suspects(end_trigger, horizon, i, is_horizon_lengthenable, true) {
		return Some(res);
	}
	match common_str_cmd(horizon, i, is_horizon_lengthenable, true) {
		CommonStrCmdResult::None => None,
		CommonStrCmdResult::Some(x) => Some(x),
		CommonStrCmdResult::OnlyWithQuotes(_) => {
			Some(WhatNow{
				tri: Transition::Push(Box::new(SitStrPhantom{
					cmd_end_trigger: end_trigger,
				})), pre: i, len: 0, alt: Some(b"\"")
			})
		}
	}
}

pub fn common_cmd_quoting_unneeded(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i, is_horizon_lengthenable) {
		return Some(res);
	}
	common_token_quoting_unneeded(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_expr_quoting_unneeded(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if horizon[i] == b'#' {
		return Some(push_comment(i));
	}
	common_token_quoting_unneeded(end_trigger, horizon, i, is_horizon_lengthenable)
}

pub fn common_token_quoting_unneeded(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	if let Some(res) = find_usual_suspects(end_trigger, horizon, i, is_horizon_lengthenable, false) {
		return Some(res);
	}
	match common_str_cmd(horizon, i, is_horizon_lengthenable, false) {
		CommonStrCmdResult::None => None,
		CommonStrCmdResult::Some(x) => Some(x),
		CommonStrCmdResult::OnlyWithQuotes(x) => {
			if let Some(replacement) = x.alt {
				if replacement.len() >= x.len {
					#[allow(clippy::collapsible_if)] // Could be expanded.
					if horizon[i] == b'`' {
						return Some(WhatNow{
							tri: Transition::Push(Box::new(SitNormal{
								end_trigger: u16::from(b'`'), end_replace: None
							})), pre: i, len: 1, alt: None
						});
					}
				}
			}
			Some(x)
		}
	}
}

// Does not pop on eof → Callers must use flush_or_pop
fn find_command_enders(
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<WhatNow> {
	let plen = prefixlen(&horizon[i..], b">&");
	if plen == 2 {
		return Some(flush(i + 2));
	}
	if i + plen == horizon.len() && (i > 0 || is_horizon_lengthenable) {
		return Some(flush(i));
	}
	let a = horizon[i];
	if a == b'\n' || a == b';' || a == b'|' || a == b'&' {
		return Some(WhatNow{
			tri: Transition::Pop, pre: i, len: 0, alt: None
		});
	}
	None
}

fn find_usual_suspects(
	end_trigger :u16,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
	quoting_needed : bool,
) -> Option<WhatNow> {
	let a = horizon[i];
	if u16::from(a) == end_trigger {
		return Some(WhatNow{
			tri: Transition::Pop, pre: i, len: 0, alt: None
		});
	}
	if a == b'\'' {
		return Some(WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'\'', color: COLOR_SQ,
			})),
			pre: i, len: 1, alt: None
		});
	}
	if a == b'\"' {
		return Some(WhatNow{
			tri: Transition::Push(Box::new(SitStrDq{})),
			pre: i, len: 1, alt: None
		});
	}
	if a == b'$' {
		if i+1 >= horizon.len() {
			if i > 0 || is_horizon_lengthenable {
				return Some(flush(i));
			}
			return None;
		}
		let b = horizon[i+1];
		if b == b'\'' {
			return Some(WhatNow{
				tri: Transition::Push(Box::new(SitStrSqEsc{})),
				pre: i, len: 2, alt: None
			});
		} else if b == b'*' {
			// $* → "$@" but not "$*" → "$@"
			let ext = Box::new(SitExtent{len: 0, color: COLOR_VAR});
			return Some(WhatNow{
				tri: Transition::Push(ext),
				pre: i, len: 2, alt: if_needed(quoting_needed, b"\"$@\"")
			});
		}
	}
	let (ate, delimiter) = find_heredoc(&horizon[i ..]);
	if i + ate == horizon.len() {
		if i > 0 || is_horizon_lengthenable {
			return Some(flush(i));
		}
	} else if !delimiter.is_empty() {
		return Some(WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: delimiter, color: COLOR_HERE}
			)),
			pre: i, len: ate, alt: None
		});
	} else if ate > 0 {
		return Some(flush(i + ate));
	}
	None
}

fn push_comment(pre: usize) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitComment{})),
		pre, len: 1, alt: None
	}
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum Tri {
	No,
	Maybe,
	Yes,
}

pub fn find_lvalue(horizon: &[u8]) -> (Tri, usize) {
	let mut ate = identifierlen(horizon);
	if ate == 0 {
		return (Tri::No, ate);
	}

	#[derive(Clone)]
	#[derive(Copy)]
	enum Lex {
		Ident,
		Brack,
		Pluss,
	}
	let mut state = Lex::Ident;

	loop {
		if ate == horizon.len() {
			return (Tri::Maybe, ate);
		}
		let byte :u8 = horizon[ate];
		ate += 1;

		// Recursion: There is now an expression_tracker() if needed.
		match (state, byte) {
			(Lex::Ident, b'=') => return (Tri::Yes, ate),
			(Lex::Pluss, b'=') => return (Tri::Yes, ate),
			(Lex::Ident, b'[') => state = Lex::Brack,
			(Lex::Brack, b']') => state = Lex::Ident,
			(Lex::Ident, b'+') => state = Lex::Pluss,
			(Lex::Ident, _) => return (Tri::No, ate),
			(Lex::Pluss, _) => return (Tri::No, ate),
			(Lex::Brack, _) => {}
		}
	}
}

fn find_heredoc(horizon: &[u8]) -> (usize, Vec<u8>) {
	let mut ate = predlen(|x| x == b'<', horizon);
	let mut found = Vec::<u8>::new();
	if ate != 2 {
		return (ate, found);
	}
	ate += predlen(|x| x == b'-', &horizon[ate ..]);
	ate += predlen(is_whitespace, &horizon[ate ..]);

	// Lex one word.
	let herein = &horizon[ate ..];
	found.reserve(herein.len());

	#[derive(Clone)]
	#[derive(Copy)]
	enum DelimiterSyntax {
		Word,
		WordEsc,
		Sq,
		Dq,
		DqEsc,
	}
	let mut state = DelimiterSyntax::Word;

	for byte_ref in herein {
		let byte: u8 = *byte_ref;
		state = match (state, byte) {
			(DelimiterSyntax::Word, b' ' ) => break,
			(DelimiterSyntax::Word, b'\n') => break,
			(DelimiterSyntax::Word, b'\t') => break,
			(DelimiterSyntax::Word, b'\\') => DelimiterSyntax::WordEsc,
			(DelimiterSyntax::Word, b'\'') => DelimiterSyntax::Sq,
			(DelimiterSyntax::Word, b'\"') => DelimiterSyntax::Dq,
			(DelimiterSyntax::Sq, b'\'') => DelimiterSyntax::Word,
			(DelimiterSyntax::Dq, b'\"') => DelimiterSyntax::Word,
			(DelimiterSyntax::Dq, b'\\') => DelimiterSyntax::DqEsc,
			(DelimiterSyntax::WordEsc, b'\n') => DelimiterSyntax::Word,
			(DelimiterSyntax::WordEsc, _) => {
				found.push(byte);
				DelimiterSyntax::Word
			}
			(DelimiterSyntax::DqEsc, b'\n') => DelimiterSyntax::Dq,
			(DelimiterSyntax::DqEsc, _) => {
				if byte != b'\"' && byte != b'\\' {
					found.push(b'\\');
				}
				found.push(byte);
				DelimiterSyntax::Dq
			}
			(_, _) => {
				found.push(byte);
				state
			}
		};
		ate += 1;
	}
	(ate, found)
}

#[test]
fn test_find_lvalue() {
	assert!(find_lvalue(b"") == (Tri::No, 0));
	assert!(find_lvalue(b"=") == (Tri::No, 0));
	assert!(find_lvalue(b"[]") == (Tri::No, 0));
	assert!(find_lvalue(b"esa") == (Tri::Maybe, 3));
	assert!(find_lvalue(b"esa+") == (Tri::Maybe, 4));
	assert!(find_lvalue(b"esa[]") == (Tri::Maybe, 5));
	assert!(find_lvalue(b"esa[]+") == (Tri::Maybe, 6));
	assert!(find_lvalue(b"esa ") == (Tri::No, 4));
	assert!(find_lvalue(b"esa]") == (Tri::No, 4));
	assert!(find_lvalue(b"esa=") == (Tri::Yes, 4));
	assert!(find_lvalue(b"esa+=") == (Tri::Yes, 5));
	assert!(find_lvalue(b"esa[]=") == (Tri::Yes, 6));
	assert!(find_lvalue(b"esa[]+=") == (Tri::Yes, 7));
}
