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
use crate::situation::push;
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
		if horizon.len() >= 4 {
			let is_emptystringtest = prefixlen(horizon, b"-z ") == 3;
			let is_nonemptystringtest = prefixlen(horizon, b"-n ") == 3;
			if is_emptystringtest || is_nonemptystringtest {
				let suggest = common_token(self.end_trigger, horizon, 3, is_horizon_lengthenable);
				if let Some(ref exciting) = suggest {
					if let Transition::Push(_) = &exciting.transition {
						let end_replace: &'static [u8] = if is_emptystringtest {
							b" = \"\""
						} else {
							b" != \"\""
						};
						return push_hiddentest(suggest, end_replace, self.end_trigger);
					} else if is_horizon_lengthenable {
						return flush(0);
					}
				}
			} else if prefixlen(horizon, b"x") == 1 {
				if let Some(mut suggest) = common_token(self.end_trigger, horizon, 1, is_horizon_lengthenable) {
					if let Transition::Push(_) = &suggest.transition {
						let transition = std::mem::replace(&mut suggest.transition, Transition::Flush);
						if let Transition::Push(state) = transition {
							let (pre, len, _) = suggest.transform;
							let progress = pre + len;
							if let Ok(found) = find_xyes_comparison(&horizon[progress ..], state) {
								if found {
									return push_xyes(self.end_trigger);
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
		} else if is_horizon_lengthenable {
			return flush(0);
		}
		become_regular_args(self.end_trigger)
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

fn become_regular_args(end_trigger :u16) -> WhatNow {
	WhatNow {
		transform: (0, 0, None),
		transition: Transition::Replace(Box::new(SitArg { end_trigger })),
	}
}

fn push_hiddentest(
	inner: Option<WhatNow>,
	end_replace: &'static [u8],
	end_trigger: u16,
) -> WhatNow {
	push(
		(0, 3, Some(b"")),
		Box::new(SitHiddenTest {
			inner,
			end_replace,
			end_trigger,
		}),
	)
}

fn push_xyes(end_trigger: u16) -> WhatNow {
	push((0, 1, Some(b"")), Box::new(SitXyes { end_trigger }))
}

struct SitHiddenTest {
	inner: Option<WhatNow>,
	end_replace: &'static [u8],
	end_trigger: u16,
}

impl Situation for SitHiddenTest {
	fn whatnow(&mut self, _horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		let initial_adventure = std::mem::replace(&mut self.inner, None);
		if let Some(mut exciting) = initial_adventure {
			exciting.transform.0 = 0;
			exciting
		} else {
			WhatNow {
				transform: (0, 0, Some(self.end_replace)),
				transition: Transition::Replace(Box::new(SitArg {
					end_trigger: self.end_trigger,
				})),
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
				return WhatNow {
					transform: (i, 1, Some(replacement)),
					transition: Transition::Replace(Box::new(SitArg {
						end_trigger: self.end_trigger,
					})),
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

#[cfg(test)]
use crate::testhelpers::*;

#[test]
fn test_sit_test() {
	sit_expect!(SitTest{end_trigger: 0u16}, b"", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b" ", &flush(0), &become_regular_args(0u16));

	sit_expect!(SitTest{end_trigger: 0u16}, b"-", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-z ", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-n ", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-z $", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-n $", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-z justkidding", &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-n justkidding", &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-z \"", &push_hiddentest(None, b"", 0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"-n \"", &push_hiddentest(None, b"", 0u16));

	sit_expect!(SitTest{end_trigger: 0u16}, b"x", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x$", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x`", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x\"$(echo)\" = ", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x\"$(echo)\" = x", &push_xyes(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x$(echo) = x", &push_xyes(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x`echo` == x", &push_xyes(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x\"$yes\" != x", &push_xyes(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"x$yes = y", &flush(0), &become_regular_args(0u16));
	sit_expect!(SitTest{end_trigger: 0u16}, b"$yes = x", &become_regular_args(0u16));
}

#[test]
fn test_has_rhs_xyes() {
	assert!(has_rhs_xyes(b" = x"));
	assert!(has_rhs_xyes(b" != x"));
	assert!(has_rhs_xyes(b" == x"));
	assert!(!has_rhs_xyes(b" = "));
	assert!(!has_rhs_xyes(b" = y"));
	assert!(!has_rhs_xyes(b"= x"));
	assert!(!has_rhs_xyes(b" =x"));
	assert!(!has_rhs_xyes(b"  x"));
	assert!(!has_rhs_xyes(b" ! x"));
}
