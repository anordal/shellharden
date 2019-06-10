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

pub struct SitExtent{
	pub len : usize,
	pub color: u32,
	pub end_insert :Option<&'static [u8]>,
}

impl Situation for SitExtent {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> ParseResult {
		if horizon.len() >= self.len {
			return Ok(WhatNow{tri: Transition::Pop, pre: self.len, len: 0, alt: self.end_insert});
		}
		self.len -= horizon.len();
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32{
		self.color
	}
}
