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
use ::situation::flush_or_pop;
use ::situation::COLOR_NORMAL;
use ::situation::COLOR_CMD;

use ::microparsers::is_whitespace;

use ::commonargcmd::keyword_or_command;
use ::commonargcmd::common_arg_cmd;

pub struct SitNormal {
	pub end_trigger :u16,
	pub end_replace :Option<&'static [u8]>,
}

impl Situation for SitNormal {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for (i, &a) in horizon.iter().enumerate() {
			if is_whitespace(a) || a == b';' || a == b'|' || a == b'&' || a == b'<' || a == b'>' {
				continue;
			}
			if a as u16 == self.end_trigger {
				return Ok(WhatNow{
					tri: Transition::Pop, pre: i, len: 1,
					alt: self.end_replace
				});
			}
			return Ok(keyword_or_command(
				self.end_trigger, &horizon, i, is_horizon_lengthenable
			));
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

pub struct SitCmd {
	pub end_trigger :u16,
}

impl Situation for SitCmd {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b' ' || a == b'\t' {
				return Ok(WhatNow{
					tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
					pre: i, len: 1, alt: None
				});
			}
			if a == b'(' {
				return Ok(WhatNow{
					tri: Transition::Pop, pre: i, len: 0, alt: None
				});
			}
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

struct SitArg {
	end_trigger :u16,
}

impl Situation for SitArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for (i, _) in horizon.iter().enumerate() {
			if let Some(res) = common_arg_cmd(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
