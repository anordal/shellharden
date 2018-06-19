/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use ::syntaxerror::UnsupportedSyntax;

pub trait Situation {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult;
	fn get_color(&self) -> u32;
}

pub enum Transition {
	Flush,
	FlushPopOnEof,
	Replace(Box<Situation>),
	Push(Box<Situation>),
	Pop,
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

pub type ParseResult = Result<WhatNow, UnsupportedSyntax>;

pub fn flush_or_pop(i: usize) -> Result<WhatNow, UnsupportedSyntax> {
	Ok(WhatNow{tri: Transition::FlushPopOnEof, pre: i, len: 0, alt: None})
}

pub const COLOR_NORMAL: u32 = 0x00000000;
pub const COLOR_BOLD  : u32 = 0x01000000;
pub const COLOR_ITALIC: u32 = 0x03000000;
