/*
 * Copyright 2021-2022 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::COLOR_NORMAL;
use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::flush_or_pop;
use crate::situation::COLOR_CMD;

use crate::commonargcmd::common_arg;
use crate::commonargcmd::common_token;
use crate::machine::expression_tracker;
use crate::microparsers::is_word;
use crate::microparsers::prefixlen;

use crate::sitcmd::SitArg;

pub struct SitTest {
	pub end_trigger :u16,
}

impl Situation for SitTest {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		if horizon.len() < 5 && is_horizon_lengthenable {
			return flush(0);
		}
		let is_emptystringtest = prefixlen(horizon, b" -z ") == 4;
		let is_nonemptystringtest = prefixlen(horizon, b" -n ") == 4;
		if is_emptystringtest || is_nonemptystringtest {
			if let Some(exciting) = common_token(self.end_trigger, horizon, 4, is_horizon_lengthenable) {
				return if let Transition::Push(_) = &exciting.tri {
					let end_replace: &'static [u8] = if is_emptystringtest {
						b" = \"\""
					} else {
						b" != \"\""
					};
					WhatNow{
						tri: Transition::Push(Box::new(SitHiddenTest{
							push: Some(exciting),
							end_replace,
							end_trigger: self.end_trigger,
						})), pre: 1, len: 4 - 1, alt: Some(b"")
					}
				} else {
					exciting
				};
			}
		} else if prefixlen(horizon, b" x") == 2 {
			if let Some(mut suggest) = common_token(self.end_trigger, horizon, 2, is_horizon_lengthenable) {
				if let Transition::Push(_) = &suggest.tri {
					let transition = std::mem::replace(&mut suggest.tri, Transition::Flush);
					if let Transition::Push(state) = transition {
						let progress = suggest.pre + suggest.len;
						if let Ok(found) = find_xyes_comparison(&horizon[progress ..], state) {
							if found {
								return WhatNow{
									tri: Transition::Push(Box::new(SitXyes{
										end_trigger: self.end_trigger,
									})), pre: 1, len: 1, alt: Some(b"")
								};
							}
							if is_horizon_lengthenable {
								return flush(0);
							}
						}
					}
				} else {
					return suggest;
				}
			}
		}
		WhatNow{
			tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
			pre: 0, len: 0, alt: None
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

struct SitHiddenTest {
	push: Option<WhatNow>,
	end_replace: &'static [u8],
	end_trigger: u16,
}

impl Situation for SitHiddenTest {
	fn whatnow(&mut self, _horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		let initial_adventure = std::mem::replace(&mut self.push, None);
		if let Some(mut exciting) = initial_adventure {
			exciting.pre = 0;
			exciting
		} else {
			WhatNow{
				tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
				pre: 0, len: 0, alt: Some(self.end_replace)
			}
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitXyes {
	end_trigger :u16,
}

impl Situation for SitXyes {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'x' {
				let mut replacement: &'static [u8] = b"\"\"";
				if i+1 < horizon.len() {
					if is_word(horizon[i+1]) {
						replacement = b"";
					}
				} else if i > 0 || is_horizon_lengthenable {
					return flush(i);
				}
				return WhatNow{
					tri: Transition::Replace(Box::new(SitArg{
						end_trigger: self.end_trigger,
					})), pre: i, len: 1, alt: Some(replacement)
				};
			}
			if let Some(res) = common_arg(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn find_xyes_comparison(horizon: &[u8], state: Box<dyn Situation>) -> Result<bool, ()> {
	let (found, exprlen) = expression_tracker(horizon, state)?;
	let after = &horizon[exprlen ..];

	Ok(found && has_rhs_xyes(after))
}

fn has_rhs_xyes(horizon: &[u8]) -> bool {
	#[derive(Clone)]
	#[derive(Copy)]
	enum Lex {
		Start,
		FirstSpace,
		Negation,
		FirstEq,
		SecondEq,
		SecondSpace,
	}
	let mut state = Lex::Start;

	for byte in horizon {
		match (state, byte) {
			(Lex::Start, b' ') => state = Lex::FirstSpace,
			(Lex::FirstSpace, b'=') => state = Lex::FirstEq,
			(Lex::FirstSpace, b'!') => state = Lex::Negation,
			(Lex::Negation, b'=') => state = Lex::SecondEq,
			(Lex::FirstEq, b'=') => state = Lex::SecondEq,
			(Lex::FirstEq, b' ') => state = Lex::SecondSpace,
			(Lex::SecondEq, b' ') => state = Lex::SecondSpace,
			(Lex::SecondSpace, b'x') => return true,
			(_, _) => break,
		}
	}
	false
}
