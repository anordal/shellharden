/*
 * Copyright 2016 - 2026 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::COLOR_KWD;
use crate::situation::Horizon;
use crate::situation::Situation;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::flush_or_pop;
use crate::situation::push;

use crate::commonargcmd::common_cmd_quoting_unneeded;
use crate::microparsers::is_whitespace;
use crate::microparsers::identifierlen;

use crate::sitrvalue::SitLvalue;

pub struct SitDeclare{
	pub end_trigger :u16,
}

impl Situation for SitDeclare {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if let Some(wn) = common_cmd_quoting_unneeded(self.end_trigger, horizon, i) {
				return wn;
			}
			if is_whitespace(a) {
				let lval_horizon = &horizon.input[i+1 ..];
				let len = identifierlen(lval_horizon);
				if len == lval_horizon.len() && (i > 0 || horizon.is_lengthenable) {
					return flush(i);
				}
				if len > 0 {
					let end_trigger = self.end_trigger;
					return push((i + 1, len, None), Box::new(SitLvalue{ end_trigger }));
				}
			}
		}
		flush_or_pop(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}
