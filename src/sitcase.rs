/*
 * Copyright 2018-2019 Andreas Nordal
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
use ::situation::COLOR_KWD;

use ::microparsers::predlen;
use ::microparsers::prefixlen;
use ::microparsers::is_whitespace;
use ::microparsers::is_word;

use ::commonargcmd::keyword_or_command;
use ::commonargcmd::common_no_cmd_quoting_unneeded;

pub struct SitIn {}

impl Situation for SitIn {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let len = predlen(&is_word, &horizon[i..]);
			if len == 0 {
				continue;
			}
			if i + len == horizon.len() && is_horizon_lengthenable {
				return Ok(flush(i));
			}
			let word = &horizon[i..i+len];
			match word {
				b"in" => {
					return Ok(WhatNow{
						tri: Transition::Replace(Box::new(SitCase{})),
						pre: i + len, len: 0, alt: None
					});
				},
				_ => {}
			}
			if let Some(res) = common_no_cmd_quoting_unneeded(
				0x100, horizon, i, is_horizon_lengthenable
			) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}

struct SitCase {}

impl Situation for SitCase {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let len = predlen(&is_word, &horizon[i..]);
			if len == 0 {
				if horizon[i] == b')' {
					return Ok(WhatNow{
						tri: Transition::Push(Box::new(SitCaseArm{})),
						pre: i, len: 1, alt: None
					});
				}
				continue;
			}
			if i + len == horizon.len() && is_horizon_lengthenable {
				return Ok(flush(i));
			}
			let word = &horizon[i..i+len];
			match word {
				b"esac" => {
					return Ok(WhatNow{
						tri: Transition::Pop, pre: i, len: 0, alt: None
					});
				},
				_ => {}
			}
			if let Some(res) = common_no_cmd_quoting_unneeded(
				0x100, horizon, i, is_horizon_lengthenable
			) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitCaseArm {}

impl Situation for SitCaseArm {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let a = horizon[i];
			if a == b';' {
				if i + 1 < horizon.len() {
					if horizon[i + 1] == b';' {
						return Ok(WhatNow{
							tri: Transition::Pop, pre: i, len: 0, alt: None
						});
					}
				} else if i > 0 || is_horizon_lengthenable {
					return Ok(flush(i));
				}
			}
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			// Premature esac: Survive and rewrite.
			let plen = prefixlen(&horizon[i..], b"esac");
			if plen == 4 {
				return Ok(WhatNow{
					tri: Transition::Pop, pre: i, len: 0, alt: Some(b";; ")
				});
			} else if i + plen == horizon.len() && (i > 0 || is_horizon_lengthenable) {
				return Ok(flush(i));
			}
			return Ok(keyword_or_command(0x100, &horizon, i, is_horizon_lengthenable));
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
