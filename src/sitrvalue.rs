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
use ::situation::flush_or_pop;
use ::situation::COLOR_NORMAL;

use ::commonargcmd::common_quoting_unneeded;
use ::commonargcmd::common_no_cmd;

pub struct SitRvalue {
	pub end_trigger :u16,
}

impl Situation for SitRvalue {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let a = horizon[i];
			if a == b' ' || a == b'\t' {
				return Ok(WhatNow{
					tri: Transition::Pop, pre: i, len: 1, alt: None
				});
			}
			if a == b'(' {
				return Ok(WhatNow{
					tri: Transition::Push(Box::new(SitArray{})),
					pre: i, len: 1, alt: None
				});
			}
			if let Some(res) = common_quoting_unneeded(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitArray {}

impl Situation for SitArray {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if let Some(res) = common_no_cmd(b')' as u16, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
