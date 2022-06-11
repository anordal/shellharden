/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::pop;
use crate::situation::COLOR_SQESC;
use crate::situation::COLOR_ESC;

use crate::sitextent::SitExtent;

pub struct SitStrSqEsc {}

impl Situation for SitStrSqEsc {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'\\' {
				let esc = Box::new(SitExtent{len: 1, color: COLOR_ESC});
				return WhatNow{tri: Transition::Push(esc), pre: i, len: 1, alt: None};
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
