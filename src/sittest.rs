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
use crate::situation::flush;
use crate::situation::COLOR_CMD;

use crate::commonargcmd::common_token;
use crate::microparsers::prefixlen;

use crate::sitcmd::SitArg;

pub struct SitTest {
	pub end_trigger :u16,
}

impl Situation for SitTest {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		if horizon.len() < 5 && is_horizon_lengthenable {
			return flush(0);
		}
		let is_emptystringtest = prefixlen(horizon, b" -z ") == 4;
		let is_nonemptystringtest = prefixlen(horizon, b" -n ") == 4;
		if is_emptystringtest || is_nonemptystringtest {
			if let Some(exciting) = common_token(self.end_trigger, horizon, 4, is_horizon_lengthenable) {
				return if let Transition::Push(_) = &exciting.tri {
					let end_replace: &'static [u8] = if is_emptystringtest {
						b" = \"\""
					} else {
						b" != \"\""
					};
					WhatNow{
						tri: Transition::Push(Box::new(SitHiddenTest{
							push: Some(exciting),
							end_replace,
							end_trigger: self.end_trigger,
						})), pre: 1, len: 4 - 1, alt: Some(b"")
					}
				} else {
					exciting
				};
			}
		}
		WhatNow{
			tri: Transition::Replace(Box::new(SitArg{end_trigger: self.end_trigger})),
			pre: 0, len: 0, alt: None
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_CMD
	}
}

struct SitHiddenTest {
	push: Option<WhatNow>,
	end_replace: &'static [u8],
	end_trigger: u16,
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
