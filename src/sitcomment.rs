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
use ::situation::flush_or_pop;
use ::situation::COLOR_CMT;

// Unlike SitUntilByte, does not swallow the end byte, and pops on eof.
pub struct SitComment {}

impl Situation for SitComment {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let a = horizon[i];
			if a == b'\n' {
				return Ok(WhatNow{
					tri: Transition::Pop, pre: i, len: 0, alt: None
				});
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_CMT
	}
}
