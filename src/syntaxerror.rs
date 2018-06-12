/*
 * Copyright 2017 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

pub struct UnsupportedSyntax {
	pub ctx: Vec<u8>,
	pub pos: usize,
	pub typ: &'static str,
	pub msg: &'static str,
}
