/*
 * Copyright 2017 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::flush;
use crate::situation::pop;
use crate::situation::Situation;
use crate::situation::WhatNow;

pub struct SitVec {
	pub terminator :Vec<u8>,
	pub color: u32,
}

impl Situation for SitVec {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		if horizon.len() < self.terminator.len() {
			if is_horizon_lengthenable {
				flush(0)
			} else {
				flush(horizon.len())
			}
		}
		else if horizon[0 .. self.terminator.len()] == self.terminator[..] {
			pop(0, self.terminator.len(), None)
		} else {
			flush(1)
		}
	}
	fn get_color(&self) -> u32 {
		self.color
	}
}
