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
use crate::situation::COLOR_SQESC;
use crate::situation::COLOR_ESC;

use crate::sitextent::push_extent;

pub struct SitStrSqEsc {}

impl Situation for SitStrSqEsc {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'\\' {
				return push_extent(COLOR_ESC, i, 2);
			}
			if a == b'\'' {
				return pop(i, 1, None);
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_SQESC
	}
}
