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

use ::microparsers::predlen;
use ::microparsers::is_whitespace;

pub struct SitUntilByte {
	pub until: u8,
	pub color: u32,
	pub end_replace :Option<&'static [u8]>,
}

impl Situation for SitUntilByte {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> ParseResult {
		let len = predlen(&|x| x != self.until, &horizon);
		return Ok(if len < horizon.len() {
			WhatNow{tri: Transition::Pop, pre: len, len: 1, alt: self.end_replace}
		} else {
			WhatNow{
				tri: if is_whitespace(self.until) {
					Transition::FlushPopOnEof
				} else {
					Transition::Flush
				}, pre: len, len: 0, alt: None
			}
		});
	}
	fn get_color(&self) -> u32{
		self.color
	}
}
