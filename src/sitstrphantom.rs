/*
 * Copyright 2017 - 2019 Andreas Nordal
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
use crate::situation::pop;

use crate::commonstrcmd::QuotingCtx;
use crate::commonstrcmd::CommonStrCmdResult;
use crate::commonstrcmd::common_str_cmd;

use crate::microparsers::predlen;
use crate::microparsers::is_word;

use crate::sitstrdq::SitStrDq;

pub struct SitStrPhantom {
	pub cmd_end_trigger: u16,
}

impl Situation for SitStrPhantom {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		let mouthful = predlen(is_phantomstringfood, horizon.input);
		if mouthful == horizon.input.len() {
			if horizon.is_lengthenable {
				return flush(0);
			}
		} else if u16::from(horizon.input[mouthful]) != self.cmd_end_trigger {
			match horizon.input[mouthful] {
				b'\"' => {
					return become_real(mouthful);
				}
				b'$' | b'`' => {
					match common_str_cmd(horizon, mouthful, QuotingCtx::Need) {
						CommonStrCmdResult::None => {}
						CommonStrCmdResult::Some(consult) |
						CommonStrCmdResult::OnlyWithQuotes(consult) => {
							match consult.transition {
								Transition::Flush | Transition::FlushPopOnEof => {
									if horizon.is_lengthenable {
										return flush(0);
									}
								}
								Transition::Pop | Transition::Replace(_) => {}
								Transition::Push(_) | Transition::Err(_) => {
									return consult;
								}
							}
						}
					}
				}
				_ => {}
			}
		}
		dutifully_end_the_string()
	}
	fn get_color(&self) -> u32 {
		0x00_ff0000
	}
}

fn is_phantomstringfood(c: u8) -> bool {
	c >= b'+' && is_word(c)
	&& c != b'?' && c != b'\\'
}

fn become_real(pre: usize) -> WhatNow {
	WhatNow {
		transform: (pre, 1, Some(b"")),
		transition: Transition::Replace(Box::new(SitStrDq::new())),
	}
}

fn dutifully_end_the_string() -> WhatNow {
	pop(0, 0, Some(b"\""))
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitcmd::SitNormal;
#[cfg(test)]
use crate::sitextent::push_extent;
#[cfg(test)]
use crate::situation::COLOR_VAR;
#[cfg(test)]
use crate::situation::push;

#[cfg(test)]
fn subject() -> SitStrPhantom {
	SitStrPhantom{cmd_end_trigger: 0}
}

#[test]
fn test_sit_strphantom() {
	let cod = dutifully_end_the_string();
	let found_cmdsub = push(
		(0, 2, None),
		Box::new(SitNormal {
			end_trigger: u16::from(b')'),
			end_replace: None,
		}),
	);
	sit_expect!(subject(), b"", &flush(0), &cod);
	sit_expect!(subject(), b"a", &flush(0), &cod);
	sit_expect!(subject(), b" ", &cod);
	sit_expect!(subject(), b"\\", &cod);
	sit_expect!(subject(), b"\'", &cod);
	sit_expect!(subject(), b"\"", &become_real(0));
	sit_expect!(subject(), b"$", &flush(0), &cod);
	sit_expect!(subject(), b"$(", &flush(0), &found_cmdsub);
	sit_expect!(subject(), b"a$", &flush(0), &cod);
	sit_expect!(subject(), b"a$(", &flush(0), &cod);
	sit_expect!(subject(), b"$\'", &cod);
	sit_expect!(subject(), b"$\"", &cod);
	sit_expect!(subject(), b"$@", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$*", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$#", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$?", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$-", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$$", &push_extent(COLOR_VAR, 0, 2));
	sit_expect!(subject(), b"$!", &push_extent(COLOR_VAR, 0, 2));
}
