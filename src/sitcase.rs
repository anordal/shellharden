/*
 * Copyright 2018 Andreas Nordal
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

use ::microparsers::predlen;
use ::microparsers::is_whitespace;
use ::microparsers::is_word;

use ::commonargcmd::keyword_or_command;
use ::commonargcmd::common_arg_cmd_array;

pub struct SitIn {
	pub end_trigger :u16,
}

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
						tri: Transition::Replace(Box::new(SitCase{
							end_trigger: self.end_trigger
						})), pre: i + len, len: 0, alt: None
					});
				},
				_ => {}
			}
			if let Some(res) = common_arg_cmd_array(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_BOLD | 0x800080
	}
}

struct SitCase {
	end_trigger :u16,
}

impl Situation for SitCase {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let len = predlen(&is_word, &horizon[i..]);
			if len == 0 {
				if horizon[i] == b')' {
					return Ok(WhatNow{
						tri: Transition::Push(Box::new(SitCaseArm{
							end_trigger: self.end_trigger
						})), pre: i, len: 1, alt: None
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
			if let Some(res) = common_arg_cmd_array(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitCaseArm {
	end_trigger :u16,
}

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
			return Ok(keyword_or_command(self.end_trigger, &horizon, i, is_horizon_lengthenable));
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
