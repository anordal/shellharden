/*
 * Copyright 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::COLOR_NORMAL;
use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;

use crate::sitcmd::SitArg;

pub struct SitHiddenTest {
	pub push: Option<WhatNow>,
	pub end_replace: &'static [u8],
	pub end_trigger: u16,
}

impl Situation for SitHiddenTest {
	fn whatnow(&mut self, _horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		let initial_adventure = std::mem::replace(&mut self.push, None);
		if let Some(mut exciting) = initial_adventure {
			exciting.pre = 0;
			exciting
		} else {
			WhatNow{
				tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
				pre: 0, len: 0, alt: Some(self.end_replace)
			}
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
