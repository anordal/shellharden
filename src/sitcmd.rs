/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Situation;
use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::ParseResult;
use ::situation::flush;
use ::situation::COLOR_NORMAL;
use ::situation::COLOR_BOLD;
use ::situation::COLOR_ITALIC;

use ::commonstrcmd::CommonStrCmdResult;
use ::commonstrcmd::common_str_cmd;

use ::microparsers::predlen;
use ::microparsers::is_whitespace;

use ::sitextent::SitExtent;
use ::sitstrdq::SitStrDq;
use ::sitstrphantom::SitStrPhantom;
use ::situntilbyte::SitUntilByte;
use ::sitvec::SitVec;

pub struct SitBeforeFirstArg {
	pub arg_cmd_data :ArgCmdData,
}

impl Situation for SitBeforeFirstArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let a = horizon[i];
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' {
				continue;
			}
			if a == b'#' {
				return Ok(WhatNow{
					tri: Transition::Push(Box::new(SitUntilByte{
						until: b'\n', color: COLOR_ITALIC | 0x20a040, end_replace: None
					})),
					pre: i, len: 1, alt: None
				});
			}
			return Ok(keyword_or_command(
				&self.arg_cmd_data, &horizon, i, is_horizon_lengthenable
			));
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn keyword_or_command(
	data :&ArgCmdData,
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
) -> WhatNow {
	let len = predlen(&|x| !is_whitespace(x), &horizon[i..]);
	if i + len == horizon.len() && is_horizon_lengthenable {
		return flush(i);
	}
	let word = &horizon[i..i+len];
	if word == b"[[" {
		return WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: vec!{b']', b']'}, color: 0x00007fff}
			)),
			pre: i, len: len, alt: None
		};
	}
	match KEYWORDS_SORTED.binary_search(&word) {
		Ok(_) => WhatNow{
			tri: Transition::Push(Box::new(SitExtent{
				len: len,
				color: COLOR_BOLD | 0x800080,
				end_insert: None
			})), pre: i, len: 0, alt: None
		},
		Err(_) => WhatNow{
			tri: Transition::Replace(Box::new(SitFirstArg{
				arg_cmd_data: data.clone(),
			})), pre: i, len: 0, alt: None
		},
	}
}

static KEYWORDS_SORTED :[&'static[u8]; 13] = [
	b"case",
	b"do",
	b"done",
	b"elif",
	b"else",
	b"esac",
	b"fi",
	b"for",
	b"if",
	b"select",
	b"then",
	b"until",
	b"while",
];

struct SitFirstArg {
	arg_cmd_data :ArgCmdData,
}

impl Situation for SitFirstArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if let Some(res) = common_arg_cmd(&self.arg_cmd_data, horizon, i, is_horizon_lengthenable) {
				return res;
			}
			if is_whitespace(horizon[i]) {
				return Ok(WhatNow{
					tri: Transition::Replace(Box::new(SitArg{
						arg_cmd_data: self.arg_cmd_data,
					})), pre: i, len: 1, alt: None
				});
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_BOLD
	}
}

struct SitArg {
	arg_cmd_data :ArgCmdData,
}

impl Situation for SitArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if let Some(res) = common_arg_cmd(&self.arg_cmd_data, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

#[derive(Clone)]
#[derive(Copy)]
pub struct ArgCmdData {
	pub end_trigger :u16,
	pub end_replace :Option<&'static [u8]>,
}

fn common_arg_cmd(
	data :&ArgCmdData,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<ParseResult> {
	let a = horizon[i];
	if a as u16 == data.end_trigger {
		return Some(Ok(WhatNow{
			tri: Transition::Pop, pre: i, len: 1,
			alt: data.end_replace
		}));
	}
	if a == b'#' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'\n', color: 0x0320a040, end_replace: None
			})),
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
	if a == b'\n' || a == b';' || a == b'|' || a == b'&' {
		return Some(Ok(WhatNow{
			tri: Transition::Replace(Box::new(SitBeforeFirstArg{
				arg_cmd_data: data.clone(),
			})), pre: i, len: 0, alt: None
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
					cmd_end_trigger: data.end_trigger,
				})), pre: i, len: 0, alt: Some(b"\"")
			}));
		},
	}
	let (ate, delimiter) = find_heredoc(&horizon[i ..]);
	if i + ate == horizon.len() {
		if is_horizon_lengthenable {
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
