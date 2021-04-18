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
use crate::situation::if_needed;
use crate::situation::COLOR_VAR;

#[derive(Clone)]
#[derive(Copy)]
enum State{
	Normal,
	Escape,
}

pub struct SitVarBrace {
	pub end_rm: bool,

	state: State,
	depth: usize,
}

impl SitVarBrace {
	pub fn new(end_rm: bool) -> SitVarBrace {
		SitVarBrace{
			end_rm,
			state: State::Normal,
			depth: 1,
		}
	}
}

impl Situation for SitVarBrace {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		for (i, c) in horizon.iter().enumerate() {
			match (self.state, c) {
				(State::Normal, b'{') => self.depth += 1,
				(State::Normal, b'}') => {
					self.depth -= 1;
					if self.depth == 0 {
						return WhatNow{
							tri: Transition::Pop,
							pre: i,
							len: 1,
							alt: if_needed(self.end_rm, b""),
						};
					}
				}
				(State::Normal, b'\\') => self.state = State::Escape,
				(State::Normal, _) => {}
				(State::Escape, _) => self.state = State::Normal,
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_VAR
	}
}
