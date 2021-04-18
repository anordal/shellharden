/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::syntaxerror::UnsupportedSyntax;

pub trait Situation {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow;
	fn get_color(&self) -> u32;
}

pub enum Transition {
	Flush,
	FlushPopOnEof,
	Replace(Box<dyn Situation>),
	Push(Box<dyn Situation>),
	Pop,
	Err(UnsupportedSyntax),
}

pub struct WhatNow {
	pub tri :Transition,
	pub pre :usize,
	pub len :usize,
	pub alt :Option<&'static [u8]>,
}

pub fn flush(i: usize) -> WhatNow {
	WhatNow{tri: Transition::Flush, pre: i, len: 0, alt: None}
}

pub fn flush_or_pop(i: usize) -> WhatNow {
	WhatNow{tri: Transition::FlushPopOnEof, pre: i, len: 0, alt: None}
}

pub fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	if needed { Some(val) } else { None }
}

pub const COLOR_NORMAL: u32 = 0x00_000000;
const COLOR_BOLD : u32 = 0x01_000000;
const COLOR_ITAL : u32 = 0x02_000000;

pub const COLOR_KWD   : u32 = COLOR_BOLD;
pub const COLOR_CMD   : u32 = 0x00_c00080;
pub const COLOR_MAGIC : u32 = 0x00_c000c0;
pub const COLOR_VAR   : u32 = 0x00_007fff;
pub const COLOR_HERE  : u32 = 0x00_802000;
pub const COLOR_CMT   : u32 = 0x00_283020 | COLOR_BOLD | COLOR_ITAL;
