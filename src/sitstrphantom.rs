/*
 * Copyright 2017 Andreas Nordal
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

use ::commonstrcmd::CommonStrCmdResult;
use ::commonstrcmd::common_str_cmd;

use ::microparsers::predlen;
use ::microparsers::is_word;

use ::sitstrdq::SitStrDq;

pub struct SitStrPhantom {
	pub cmd_end_trigger: u16,
}

impl Situation for SitStrPhantom {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		let mouthful = predlen(&is_phantomstringfood, &horizon);
		if mouthful == horizon.len() {
			if is_horizon_lengthenable {
				return Ok(flush(0));
			}
		} else if u16::from(horizon[mouthful]) != self.cmd_end_trigger {
			match horizon[mouthful] {
				b'\"' => {
					return become_real(mouthful);
				}
				b'$' | b'`' => {
					match common_str_cmd(&horizon, mouthful, is_horizon_lengthenable, true) {
						CommonStrCmdResult::None => {},
						CommonStrCmdResult::Err(e) => { return Err(e); },
						CommonStrCmdResult::Ok(consult) |
						CommonStrCmdResult::OnlyWithQuotes(consult) => {
							match consult.tri {
								Transition::Flush | Transition::FlushPopOnEof => {
									if is_horizon_lengthenable {
										return Ok(flush(0));
									}
								}
								Transition::Pop | Transition::Replace(_) => {}
								Transition::Push(_) => {
									return Ok(consult);
								}
							}
						},
					}
				}
				_ => {}
			}
		}
		dutifully_end_the_string()
	}
	fn get_color(&self) -> u32{
		0x00_ff0000
	}
}

fn is_phantomstringfood(c: u8) -> bool {
	c >= b'+' && is_word(c)
	&& c != b'?' && c != b'\\'
}

fn become_real(pre: usize) -> ParseResult {
	Ok(WhatNow{
		tri: Transition::Replace(Box::new(SitStrDq{})),
		pre, len: 1, alt: Some(b"")
	})
}

fn dutifully_end_the_string() -> ParseResult {
	Ok(WhatNow{
		tri: Transition::Pop, pre: 0, len: 0, alt: Some(b"\"")
	})
}

#[cfg(test)]
use ::testhelpers::*;
#[cfg(test)]
use sitcmd::SitNormal;
#[cfg(test)]
use sitextent::SitExtent;
#[cfg(test)]
use ::situation::COLOR_VAR;

#[cfg(test)]
fn subject() -> SitStrPhantom {
	SitStrPhantom{cmd_end_trigger: 0}
}

#[test]
fn test_sit_strphantom() {
	let cod = dutifully_end_the_string();
	let found_cmdsub = Ok(WhatNow{
		tri: Transition::Push(Box::new(SitNormal{
			end_trigger: u16::from(b')'), end_replace: None,
		})), pre: 0, len: 2, alt: None
	});
	let found_specialvar = Ok(WhatNow{
		tri: Transition::Push(Box::new(SitExtent{
			len: 2, color: COLOR_VAR, end_insert: None,
		})), pre: 0, len: 0, alt: None
	});
	sit_expect!(subject(), b"", &Ok(flush(0)), &cod);
	sit_expect!(subject(), b"a", &Ok(flush(0)), &cod);
	sit_expect!(subject(), b" ", &cod);
	sit_expect!(subject(), b"\'", &cod);
	sit_expect!(subject(), b"\"", &become_real(0));
	sit_expect!(subject(), b"$", &Ok(flush(0)), &cod);
	sit_expect!(subject(), b"$(", &Ok(flush(0)), &found_cmdsub);
	sit_expect!(subject(), b"a$", &Ok(flush(0)), &cod);
	sit_expect!(subject(), b"a$(", &Ok(flush(0)), &cod);
	sit_expect!(subject(), b"$\'", &cod);
	sit_expect!(subject(), b"$\"", &cod);
	sit_expect!(subject(), b"$@", &found_specialvar);
	sit_expect!(subject(), b"$*", &found_specialvar);
	sit_expect!(subject(), b"$#", &found_specialvar);
	sit_expect!(subject(), b"$?", &found_specialvar);

	// TODO: Unimplemented special variables
	sit_expect!(subject(), b"$-", &cod);
	sit_expect!(subject(), b"$$", &cod);
	sit_expect!(subject(), b"$!", &cod);
}
