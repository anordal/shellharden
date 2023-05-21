/*
 * Copyright 2016 - 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

pub fn prefixlen(a: &[u8], b: &[u8]) -> usize {
	let mut i: usize = 0;
	while i < a.len() && i < b.len() && a[i] == b[i] {
		i += 1;
	}
	i
}

pub fn predlen(pred: impl Fn(u8) -> bool, horizon: &[u8]) -> usize {
	let mut i: usize = 0;
	while i < horizon.len() && pred(horizon[i]) {
		i += 1;
	}
	i
}

pub fn is_identifierhead(c: u8) -> bool {
	matches!(c, b'a' ..= b'z' | b'A' ..= b'Z' | b'_')
}

pub fn is_identifiertail(c: u8) -> bool {
	matches!(c, b'a' ..= b'z' | b'A' ..= b'Z' | b'0' ..= b'9' | b'_')
}

pub fn identifierlen(horizon: &[u8]) -> usize {
	if !horizon.is_empty() && is_identifierhead(horizon[0]) {
		1 + predlen(is_identifiertail, &horizon[1 ..])
	} else {
		0
	}
}

pub fn is_whitespace(c: u8) -> bool {
	c <= b' '
}

pub fn is_lowercase(c: u8) -> bool {
	matches!(c, b'a' ..= b'z')
}

pub fn is_word(byte: u8) -> bool {
	!matches!(byte, 0 ..= b' ' | b'&' | b'(' | b')' | b';' | b'<' | b'>' | b'`' | b'|')
}
