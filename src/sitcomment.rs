/*
 * Copyright 2018 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Horizon;
use crate::situation::Situation;
use crate::situation::WhatNow;
use crate::situation::flush_or_pop;
use crate::situation::COLOR_CMT;
use crate::situation::pop;

pub struct SitComment {}

impl Situation for SitComment {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if a == b'\n' {
				return pop(i, 0, None);
			}
		}
		flush_or_pop(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_CMT
	}
}
