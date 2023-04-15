/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::pop;
use crate::situation::push;

pub struct SitExtent{
	pub len: usize,
	pub color: u32,
}

impl Situation for SitExtent {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		if horizon.len() >= self.len {
			return pop(self.len, 0, None);
		}
		self.len -= horizon.len();
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		self.color
	}
}

pub fn push_extent(color: u32, pre: usize, len: usize) -> WhatNow {
	push((pre, 0, None), Box::new(SitExtent { len, color }))
}

pub fn push_replaceable(color: u32, pre: usize, len: usize, alt: Option<&'static [u8]>) -> WhatNow {
	push((pre, len, alt), Box::new(SitExtent { len: 0, color }))
}
