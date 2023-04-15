/*
 * Copyright 2016 - 2022 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::pop;

use crate::commonstrcmd::QuotingCtx;
use crate::commonstrcmd::CommonStrCmdResult;
use crate::commonstrcmd::common_str_cmd;

pub struct SitStrDq {
	interpolation_detection: QuotingCtx,
}

impl SitStrDq {
	pub fn new() -> SitStrDq {
		SitStrDq{ interpolation_detection: QuotingCtx::Dontneed }
	}
}

impl Situation for SitStrDq {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for (i, &a) in horizon.iter().enumerate() {
			if a == b'\"' {
				return pop(i, 1, None);
			}
			match common_str_cmd(horizon, i, is_horizon_lengthenable, self.interpolation_detection) {
				CommonStrCmdResult::None => {
					self.interpolation_detection = QuotingCtx::Interpolation;
				}
				CommonStrCmdResult::Some(x) |
				CommonStrCmdResult::OnlyWithQuotes(x) => {
					let (pre, len, _) = x.transform;
					let progress = pre + len;
					if progress != 0 {
						self.interpolation_detection = QuotingCtx::Interpolation;
					}
					return x;
				}
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		0x00_ff0000
	}
}

#[cfg(test)]
use crate::testhelpers::*;
#[cfg(test)]
use crate::sitcmd::SitNormal;
#[cfg(test)]
use crate::sitextent::push_extent;
#[cfg(test)]
use crate::sitmagic::push_magic;
#[cfg(test)]
use crate::situation::push;
#[cfg(test)]
use crate::situation::COLOR_ESC;

#[test]
fn test_sit_strdq() {
	let found_cmdsub = push(
		(0, 2, None),
		Box::new(SitNormal {
			end_trigger: u16::from(b')'),
			end_replace: None,
		}),
	);
	sit_expect!(SitStrDq::new(), b"", &flush(0));
	sit_expect!(SitStrDq::new(), b"$", &flush(0), &flush(1));
	sit_expect!(SitStrDq::new(), b"$(", &flush(0), &found_cmdsub);
	sit_expect!(SitStrDq::new(), b"$( ", &found_cmdsub);
	sit_expect!(SitStrDq::new(), b"$((", &push_magic(0, 2, b')'));
	sit_expect!(SitStrDq::new(), b"\\", &push_extent(COLOR_ESC, 0, 2));
}
