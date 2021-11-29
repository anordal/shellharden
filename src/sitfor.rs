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
use crate::situation::COLOR_KWD;
use crate::situation::COLOR_VAR;
use crate::situation::COLOR_NORMAL;

use crate::microparsers::identifierlen;
use crate::microparsers::is_word;
use crate::microparsers::is_identifierhead;
use crate::microparsers::is_identifiertail;
use crate::microparsers::is_whitespace;
use crate::microparsers::predlen;

use crate::sitvarident::SitVarIdent;
use crate::commonargcmd::common_arg;

pub struct SitFor {}

impl Situation for SitFor {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			let len = predlen(is_word, &horizon[i..]);
			if i + len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
				return flush(i);
			}
			let word = &horizon[i..i+len];
			if word == b"in" {
				return WhatNow{
					tri: Transition::Push(Box::new(SitForIn{})),
					pre: i, len, alt: None
				};
			}
			if is_identifierhead(a) {
				return WhatNow{
					tri: Transition::Push(Box::new(SitVarIdent{end_insert: None})),
					pre: i, len: 1, alt: None
				};
			}
			if !is_whitespace(a) {
				return WhatNow{
					tri: Transition::Pop,
					pre: i, len: 0, alt: None,
				};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_KWD
	}
}

pub struct SitForIn {}

impl Situation for SitForIn {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'$' {
				let idlen = identifierlen(&horizon[i+1 ..]);
				let len = idlen + predlen(|x| x == b' ', &horizon[i+1+idlen ..]);
				if i+1+len == horizon.len() && (i > 0 || is_horizon_lengthenable) {
					return flush(i);
				}
				if idlen >= 1 && matches!(horizon[i+1+len], b';' | b'\n') {
					return WhatNow{
						tri: Transition::Replace(Box::new(SitVarIdentNecessarilyArray{})),
						pre: i, len: 1, alt: Some(b"\"${"),
					};
				}
			}
			if !is_whitespace(a) {
				return WhatNow{
					tri: Transition::Replace(Box::new(SitForInAnythingElse{})),
					pre: i, len: 0, alt: None,
				};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitVarIdentNecessarilyArray {}

impl Situation for SitVarIdentNecessarilyArray {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			// An identifierhead is also an identifiertail.
			if !is_identifiertail(a) {
				return WhatNow{
					tri: Transition::Pop,
					pre: i, len: 0, alt: Some(b"[@]}\""),
				};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_VAR
	}
}

pub struct SitForInAnythingElse {}

impl Situation for SitForInAnythingElse {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if let Some(res) = common_arg(u16::from(b';'), horizon, i, is_horizon_lengthenable) {
				return res;
			}
			if a == b'\n' {
				return WhatNow{
					tri: Transition::Pop,
					pre: i, len: 1, alt: None,
				};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}
