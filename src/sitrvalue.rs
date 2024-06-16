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
use crate::situation::Transition;
use crate::situation::pop;
use crate::situation::push;
use crate::situation::flush;
use crate::situation::flush_or_pop;
use crate::situation::COLOR_NORMAL;
use crate::situation::COLOR_LVAL;

use crate::microparsers::is_whitespace;

use crate::commonargcmd::common_cmd_quoting_unneeded;
use crate::commonargcmd::common_expr;

pub struct SitLvalue {
	pub len :usize,
	pub end_trigger :u16,
}

impl Situation for SitLvalue {
	fn whatnow(&mut self, _: Horizon) -> WhatNow {
		WhatNow {
			transform: (self.len, 1, None),
			transition: Transition::Replace(Box::new(SitRvalue{ end_trigger: self.end_trigger })),
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_LVAL
	}
}

struct SitRvalue {
	end_trigger :u16,
}

impl Situation for SitRvalue {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, &a) in horizon.input.iter().enumerate() {
			if a == b'(' {
				return push((i, 1, None), Box::new(SitArray {}));
			}
			if let Some(res) = common_cmd_quoting_unneeded(self.end_trigger, horizon, i) {
				return res;
			}
			if is_whitespace(a) {
				return pop(i, 1, None);
			}
		}
		flush_or_pop(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitArray {}

impl Situation for SitArray {
	fn whatnow(&mut self, horizon: Horizon) -> WhatNow {
		for (i, _) in horizon.input.iter().enumerate() {
			if let Some(res) = common_expr(u16::from(b')'), horizon, i) {
				return res;
			}
		}
		flush(horizon.input.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
