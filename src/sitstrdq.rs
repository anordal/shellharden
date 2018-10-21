/*
 * Copyright 2016 Andreas Nordal
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

use commonstrcmd::CommonStrCmdResult;
use commonstrcmd::common_str_cmd;

pub struct SitStrDq {}

impl Situation for SitStrDq {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\"' {
				return Ok(WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None});
			}
			match common_str_cmd(&horizon, i, is_horizon_lengthenable, false) {
				CommonStrCmdResult::None => {},
				CommonStrCmdResult::Err(e) => { return Err(e); },
				CommonStrCmdResult::Ok(x) => { return Ok(x); },
				CommonStrCmdResult::OnlyWithQuotes(x) => { return Ok(x); },
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32{
		0x00ff0000
	}
}
