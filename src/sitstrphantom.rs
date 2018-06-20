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
		} else if horizon[mouthful] as u16 != self.cmd_end_trigger {
			match horizon[mouthful] {
				b'\"' => {
					return Ok(WhatNow{
						tri: Transition::Replace(Box::new(SitStrDq{})),
						pre: mouthful, len: 1, alt: Some(b"")
					});
				}
				b'$' | b'`' => {
					match common_str_cmd(&horizon, mouthful, is_horizon_lengthenable, true) {
						CommonStrCmdResult::None => {},
						CommonStrCmdResult::Err(e) => { return Err(e); },
						CommonStrCmdResult::Ok(consult) |
						CommonStrCmdResult::OnlyWithQuotes(consult) => {
							match &consult.tri {
								&Transition::Flush | &Transition::FlushPopOnEof => {
									return Ok(WhatNow{
										tri: Transition::FlushPopOnEof,
										pre: 0, len: 0, alt: Some(b"\"")
									});
								}
								&Transition::Pop | &Transition::Replace(_) => {}
								&Transition::Push(_) => {
									return Ok(consult);
								}
							}
						},
						CommonStrCmdResult::OnlyWithoutQuotes(_) => {},
					}
				}
				_ => {}
			}
		}
		// Dutifully end the string.
		return Ok(WhatNow{
			tri: Transition::Pop, pre: 0, len: 0, alt: Some(b"\"")
		});
	}
	fn get_color(&self) -> u32{
		0x00ff0000
	}
}

fn is_phantomstringfood(c: u8) -> bool {
	c >= b'+' && is_word(c)
	&& c != b'?' && c != b'\\'
}
