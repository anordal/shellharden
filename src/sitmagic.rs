/*
 * Copyright 2021 Andreas Nordal
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
use crate::situation::COLOR_MAGIC;

use crate::commonargcmd::common_token_quoting_unneeded;

// Magic syntax (as opposed to builtin commands)
pub struct SitMagic {
	pub end_trigger :u8,
}

impl Situation for SitMagic {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'(' {
				return push_magic(i, 1, b')');
			}
			if a == b'[' {
				return push_magic(i, 1, b']');
			}
			if a == self.end_trigger {
				return pop(i, 1, None);
			}
			if let Some(res) = common_token_quoting_unneeded(0x100, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_MAGIC
	}
}

pub fn push_magic(pre: usize, len: usize, end_trigger: u8) -> WhatNow {
	WhatNow{
		tri: Transition::Push(Box::new(SitMagic{end_trigger})),
		pre, len, alt: None
	}
}
