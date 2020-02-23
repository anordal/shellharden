/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::situation::Situation;
use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::flush;

use commonstrcmd::CommonStrCmdResult;
use commonstrcmd::common_str_cmd;

pub struct SitStrDq {}

impl Situation for SitStrDq {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'\"' {
				return WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None};
			}
			match common_str_cmd(&horizon, i, is_horizon_lengthenable, false) {
				CommonStrCmdResult::None => {}
				CommonStrCmdResult::Some(x) |
				CommonStrCmdResult::OnlyWithQuotes(x) => { return x; }
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		0x00_ff0000
	}
}

#[cfg(test)]
use ::testhelpers::*;
#[cfg(test)]
use sitcmd::SitNormal;
#[cfg(test)]
use sitvec::SitVec;
#[cfg(test)]
use situation::COLOR_MAGIC;

#[test]
fn test_sit_strdq() {
	let found_cmdsub = WhatNow{
		tri: Transition::Push(Box::new(SitNormal{
			end_trigger: u16::from(b')'), end_replace: None,
		})), pre: 0, len: 2, alt: None
	};
	let found_math = WhatNow{
		tri: Transition::Push(Box::new(SitVec{
			terminator: vec!{b')', b')'},
			color: COLOR_MAGIC,
		})), pre: 0, len: 3, alt: None
	};
	sit_expect!(SitStrDq{}, b"", &flush(0));
	sit_expect!(SitStrDq{}, b"$", &flush(0), &flush(1));
	sit_expect!(SitStrDq{}, b"$(", &flush(0), &found_cmdsub);
	sit_expect!(SitStrDq{}, b"$( ", &found_cmdsub);
	sit_expect!(SitStrDq{}, b"$((", &found_math);
}
