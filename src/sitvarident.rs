/*
 * Copyright 2017 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::WhatNow;
use crate::situation::COLOR_VAR;

use crate::microparsers::predlen;
use crate::microparsers::is_identifiertail;

pub struct SitVarIdent {
	pub end_insert: Option<&'static [u8]>,
}

impl Situation for SitVarIdent {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		let len = predlen(is_identifiertail, &horizon);
		if len < horizon.len() {
			return WhatNow{tri: Transition::Pop, pre: len, len: 0, alt: self.end_insert};
		}
		WhatNow{
			tri: Transition::FlushPopOnEof,
			pre: horizon.len(), len: 0, alt: self.end_insert
		}
	}
	fn get_color(&self) -> u32 {
		COLOR_VAR
	}
}
