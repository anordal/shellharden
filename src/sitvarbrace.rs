/*
 * Copyright 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::situation::Situation;
use crate::situation::WhatNow;
use crate::situation::flush;
use crate::situation::if_needed;
use crate::situation::pop;
use crate::situation::COLOR_VAR;

use crate::sitextent::push_extent;

#[derive(Clone)]
#[derive(Copy)]
enum State{
	Name,
	Index,
	Normal,
	Escape,
}

pub struct SitVarBrace {
	end_rm: bool,
	state: State,
	depth: usize,
}

impl SitVarBrace {
	pub fn new(end_rm: bool, replace_s11n: bool) -> SitVarBrace {
		SitVarBrace{
			end_rm,
			state: if replace_s11n { State::Name } else { State::Normal },
			depth: 1,
		}
	}
}

impl Situation for SitVarBrace {
	fn whatnow(&mut self, horizon: &[u8], _is_horizon_lengthenable: bool) -> WhatNow {
		for (i, c) in horizon.iter().enumerate() {
			match (self.state, c) {
				(State::Name, b'a' ..= b'z') |
				(State::Name, b'A' ..= b'Z') |
				(State::Name, b'0' ..= b'9') |
				(State::Name, b'_') => {}
				(State::Name, b'[') => self.state = State::Index,
				(State::Index, b'*') => {
					self.state = State::Normal;
					return push_extent(COLOR_VAR, i, 1, Some(b"@"));
				}
				(State::Normal, b'{') => self.depth += 1,
				(State::Name | State::Index | State::Normal, b'}') => {
					self.depth -= 1;
					if self.depth == 0 {
						return pop(i, 1, if_needed(self.end_rm, b""));
					}
				}
				(State::Name, _) => self.state = State::Normal,
				(State::Index, _) => self.state = State::Normal,
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
