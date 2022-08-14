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
	pub transform: (usize, usize, Option<&'static [u8]>), // pre, len, alt
	pub transition: Transition,
}

pub fn flush(pre: usize) -> WhatNow {
	WhatNow {
		transform: (pre, 0, None),
		transition: Transition::Flush,
	}
}

pub fn flush_or_pop(pre: usize) -> WhatNow {
	WhatNow {
		transform: (pre, 0, None),
		transition: Transition::FlushPopOnEof,
	}
}

pub fn pop(pre: usize, len: usize, alt: Option<&'static [u8]>) -> WhatNow {
	WhatNow {
		transform: (pre, len, alt),
		transition: Transition::Pop,
	}
}

pub fn push(transform: (usize, usize, Option<&'static [u8]>), sit: Box<dyn Situation>) -> WhatNow {
	WhatNow {
		transform,
		transition: Transition::Push(sit),
	}
}

pub fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	if needed { Some(val) } else { None }
}

pub const COLOR_NORMAL: u32 = 0x00_000000;
const COLOR_BOLD : u32 = 0x01_000000;
const COLOR_ITAL : u32 = 0x02_000000;
const COLOR_GOLD : u32 = 0x00_ffcc55;

pub const COLOR_KWD   : u32 = COLOR_BOLD;
pub const COLOR_CMD   : u32 = 0x00_c00080;
pub const COLOR_MAGIC : u32 = 0x00_c000c0;
pub const COLOR_VAR   : u32 = 0x00_007fff;
pub const COLOR_HERE  : u32 = 0x00_802000;
pub const COLOR_CMT   : u32 = 0x00_789060 | COLOR_BOLD | COLOR_ITAL;
pub const COLOR_SQ    : u32 = COLOR_GOLD;
pub const COLOR_ESC   : u32 = 0x00_ff0080 | COLOR_BOLD;
pub const COLOR_SQESC : u32 = 0x00_ff8000;
