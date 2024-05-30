/*
 * Copyright 2016 - 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Horizon;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::if_needed;
use crate::situation::pop;
use crate::situation::push;
use crate::situation::COLOR_HERE;
use crate::situation::COLOR_KWD;
use crate::situation::COLOR_SQ;
use crate::situation::COLOR_VAR;

use crate::microparsers::prefixlen;
use crate::microparsers::predlen;
use crate::microparsers::identifierlen;
use crate::microparsers::is_whitespace;
use crate::microparsers::is_word;

use crate::commonstrcmd::QuotingCtx;
use crate::commonstrcmd::CommonStrCmdResult;
use crate::commonstrcmd::common_str_cmd;

use crate::sitcase::SitCase;
use crate::sitfor::SitFor;
use crate::sitcmd::SitNormal;
use crate::sitcmd::SitCmd;
use crate::sitcomment::SitComment;
use crate::sitextent::push_extent;
use crate::sitextent::push_replaceable;
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
	horizon: Horizon,
	i: usize,
) -> WhatNow {
	if horizon.input[i] == b'#' {
		return push_comment(i);
	}
	let (found, len) = find_lvalue(&horizon.input[i..]);
	if found == Tri::Maybe && (i > 0 || horizon.is_lengthenable) {
		return flush(i);
	}
	if found == Tri::Yes {
		return push((i + len, 0, None), Box::new(SitRvalue { end_trigger }));
	}
	let len = predlen(is_word, &horizon.input[i..]);
	let len = if len != 0 { len } else { prefixlen(&horizon.input[i..], b"((") };
	if i + len == horizon.input.len() && (i > 0 || horizon.is_lengthenable) {
		return flush(i);
	}
	let word = &horizon.input[i..i+len];
	match word {
		b"(" => push(
			(i, 1, None),
			Box::new(SitNormal {
				end_trigger: u16::from(b')'),
				end_replace: None,
			}),
		),
		b"((" => push_magic(i, 1, b')'),
		b"[[" => push_magic(i, 1, b']'),
		b"case" => push((i, len, None), Box::new(SitCase {})),
		b"for" |
		b"select" => push((i, len, None), Box::new(SitFor {})),
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
		b"}" => push_extent(COLOR_KWD, i, len),
		b"[" |
		b"test" if predlen(|x| x == b' ', &horizon.input[i + len ..]) == 1 => {
			push((i, len + 1, None), Box::new(SitTest { end_trigger }))
		},
		_ => push((i, 0, None), Box::new(SitCmd { end_trigger })),
	}
}

pub fn common_arg(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i) {
		return Some(res);
	}
	common_expr(end_trigger, horizon, i)
}

pub fn common_cmd(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i) {
		return Some(res);
	}
	common_token(end_trigger, horizon, i)
}

pub fn common_expr(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if horizon.input[i] == b'#' {
		return Some(push_comment(i));
	}
	common_token(end_trigger, horizon, i)
}

pub fn common_token(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if let Some(res) = find_usual_suspects(end_trigger, horizon, i, true) {
		return Some(res);
	}
	match common_str_cmd(horizon, i, QuotingCtx::Need) {
		CommonStrCmdResult::None => None,
		CommonStrCmdResult::Some(x) => Some(x),
		CommonStrCmdResult::OnlyWithQuotes(_) => Some(push(
			(i, 0, Some(b"\"")),
			Box::new(SitStrPhantom {
				cmd_end_trigger: end_trigger,
			}),
		)),
	}
}

pub fn common_cmd_quoting_unneeded(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if let Some(res) = find_command_enders(horizon, i) {
		return Some(res);
	}
	common_token_quoting_unneeded(end_trigger, horizon, i)
}

pub fn common_expr_quoting_unneeded(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if horizon.input[i] == b'#' {
		return Some(push_comment(i));
	}
	common_token_quoting_unneeded(end_trigger, horizon, i)
}

pub fn common_token_quoting_unneeded(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	if let Some(res) = find_usual_suspects(end_trigger, horizon, i, false) {
		return Some(res);
	}
	match common_str_cmd(horizon, i, QuotingCtx::Dontneed) {
		CommonStrCmdResult::None => None,
		CommonStrCmdResult::Some(x) => Some(x),
		CommonStrCmdResult::OnlyWithQuotes(x) => {
			let (_, len, alt) = x.transform;
			if let Some(replacement) = alt {
				if replacement.len() >= len {
					#[allow(clippy::collapsible_if)] // Could be expanded.
					if horizon.input[i] == b'`' {
						return Some(push(
							(i, 1, None),
							Box::new(SitNormal {
								end_trigger: u16::from(b'`'),
								end_replace: None,
							}),
						));
					}
				}
			}
			Some(x)
		}
	}
}

// Does not pop on eof → Callers must use flush_or_pop
fn find_command_enders(
	horizon :Horizon,
	i :usize,
) -> Option<WhatNow> {
	let plen = prefixlen(&horizon.input[i..], b">&");
	if plen == 2 {
		return Some(flush(i + 2));
	}
	if i + plen == horizon.input.len() && (i > 0 || horizon.is_lengthenable) {
		return Some(flush(i));
	}
	let a = horizon.input[i];
	if a == b'\n' || a == b';' || a == b'|' || a == b'&' {
		return Some(pop(i, 0, None));
	}
	None
}

fn find_usual_suspects(
	end_trigger :u16,
	horizon :Horizon,
	i :usize,
	quoting_needed : bool,
) -> Option<WhatNow> {
	let a = horizon.input[i];
	if u16::from(a) == end_trigger {
		return Some(pop(i, 0, None));
	}
	if a == b'\'' {
		return Some(push(
			(i, 1, None),
			Box::new(SitUntilByte {
				until: b'\'',
				color: COLOR_SQ,
			}),
		));
	}
	if a == b'\"' {
		return Some(push((i, 1, None), Box::new(SitStrDq::new())));
	}
	if a == b'$' {
		if i+1 >= horizon.input.len() {
			if i > 0 || horizon.is_lengthenable {
				return Some(flush(i));
			}
			return None;
		}
		let b = horizon.input[i+1];
		if b == b'\'' {
			return Some(push((i, 2, None), Box::new(SitStrSqEsc {})));
		} else if b == b'*' {
			// $* → "$@" but not "$*" → "$@"
			return Some(push_replaceable(COLOR_VAR, i, 2, if_needed(quoting_needed, b"\"$@\"")));
		}
	}
	let (ate, delimiter) = find_heredoc(&horizon.input[i ..]);
	if i + ate == horizon.input.len() {
		if i > 0 || horizon.is_lengthenable {
			return Some(flush(i));
		}
	} else if !delimiter.is_empty() {
		return Some(push(
			(i, ate, None),
			Box::new(SitVec {
				terminator: delimiter,
				color: COLOR_HERE,
			}),
		));
	} else if ate > 0 {
		return Some(flush(i + ate));
	}
	None
}

fn push_comment(pre: usize) -> WhatNow {
	push((pre, 1, None), Box::new(SitComment {}))
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
	assert!(find_lvalue(b"esa+  ") == (Tri::No, 5));
	assert!(find_lvalue(b"esa[]") == (Tri::Maybe, 5));
	assert!(find_lvalue(b"esa[]+") == (Tri::Maybe, 6));
	assert!(find_lvalue(b"esa ") == (Tri::No, 4));
	assert!(find_lvalue(b"esa]") == (Tri::No, 4));
	assert!(find_lvalue(b"esa=") == (Tri::Yes, 4));
	assert!(find_lvalue(b"esa+=") == (Tri::Yes, 5));
	assert!(find_lvalue(b"esa[]=") == (Tri::Yes, 6));
	assert!(find_lvalue(b"esa[]+=") == (Tri::Yes, 7));
}
