/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Situation;
use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::flush;

use ::microparsers::predlen;

pub struct SitUntilByte {
	pub until: u8,
	pub color: u32,
	pub end_replace :Option<&'static [u8]>,
}

impl Situation for SitUntilByte {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		let len = predlen(&|x| x != self.until, &horizon);
		if len < horizon.len() {
			WhatNow{tri: Transition::Pop, pre: len, len: 1, alt: self.end_replace}
		} else {
			flush(len)
		}
	}
	fn get_color(&self) -> u32{
		self.color
	}
}
