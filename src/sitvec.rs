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

pub struct SitVec {
	pub terminator :Vec<u8>,
	pub color: u32,
}

impl Situation for SitVec {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		if horizon.len() < self.terminator.len() {
			if is_horizon_lengthenable {
				Ok(flush(0))
			} else {
				Ok(flush(horizon.len()))
			}
		}
		else if &horizon[0 .. self.terminator.len()] == &self.terminator[..] {
			Ok(WhatNow{tri: Transition::Pop, pre: 0, len: self.terminator.len(), alt: None})
		} else {
			Ok(flush(1))
		}
	}
	fn get_color(&self) -> u32{
		self.color
	}
}
