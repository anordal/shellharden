/*
 * Copyright 2018 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::flush_or_pop;
use crate::situation::COLOR_NORMAL;

use crate::microparsers::is_whitespace;

use crate::commonargcmd::common_cmd_quoting_unneeded;
use crate::commonargcmd::common_expr;

pub struct SitRvalue {
	pub end_trigger :u16,
}

impl Situation for SitRvalue {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'(' {
				return WhatNow{
					tri: Transition::Push(Box::new(SitArray{})),
					pre: i, len: 1, alt: None
				};
			}
			if let Some(res) = common_cmd_quoting_unneeded(self.end_trigger, horizon, i, is_horizon_lengthenable) {
				return res;
			}
			if is_whitespace(a) {
				return WhatNow{
					tri: Transition::Pop, pre: i, len: 1, alt: None
				};
			}
		}
		flush_or_pop(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitArray {}

impl Situation for SitArray {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, _) in horizon.iter().enumerate() {
			if let Some(res) = common_expr(u16::from(b')'), horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
