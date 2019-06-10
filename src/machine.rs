/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std;
use std::io;

use ::syntaxerror::UnsupportedSyntax;

use ::filestream::InputSource;
use ::filestream::FileOut;

use ::situation::Situation;
use ::situation::Transition;
use ::situation::WhatNow;
use ::situation::COLOR_NORMAL;

use ::sitcmd::SitNormal;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
pub enum OutputSelector {
	ORIGINAL,
	DIFF,
	TRANSFORM,
	CHECK,
}

pub struct Settings {
	pub osel :OutputSelector,
	pub syntax :bool,
	pub replace :bool,
}

pub enum Error {
	Stdio(std::io::Error),
	Syntax(UnsupportedSyntax),
	Check,
}

pub fn treatfile(path: &std::ffi::OsString, sett: &Settings) -> Result<(), Error> {
	let stdout = io::stdout(); // TODO: not here
	let mut fo: FileOut;
	{
		let stdin = io::stdin(); // TODO: not here
		let mut fi: InputSource = if path.is_empty() {
			InputSource::open_stdin(&stdin)
		} else {
			try!(InputSource::open_file(path).map_err(Error::Stdio))
		};

		fo = if sett.osel == OutputSelector::CHECK {
			FileOut::open_none()
		} else if sett.replace && !path.is_empty() {
			FileOut::open_soak(try!(fi.size().map_err(Error::Stdio)) * 9 / 8)
		} else {
			FileOut::open_stdout(&stdout)
		};

		const BUFSIZE :usize = 128;
		let mut fill :usize = 0;
		let mut buf = [0; BUFSIZE];

		let mut state :Vec<Box<Situation>> = vec!{Box::new(SitNormal{
			end_trigger: 0x100, end_replace: None,
		})};

		loop {
			let bytes = try!(fi.read(&mut buf[fill ..]).map_err(Error::Stdio));
			fill += bytes;
			let eof = bytes == 0;
			let consumed = try!(stackmachine(&mut state, &mut fo, &buf[0 .. fill], eof, &sett));
			let remain = fill - consumed;
			if eof {
				assert!(remain == 0);
				break;
			}
			if fo.change && sett.osel == OutputSelector::CHECK {
				return Err(Error::Check);
			}
			for i in 0 .. remain {
				buf[i] = buf[consumed + i];
			}
			fill = remain;
		}
		if state.len() != 1 {
			return Err(Error::Syntax(UnsupportedSyntax{
				typ: "Unexpected end of file",
				ctx: buf[0 .. fill].to_owned(),
				pos: fill,
				msg: "The file's end was reached without closing all sytactic scopes.\n\
				Either, the parser got lost, or the file is truncated or malformed.",
			}));
		}
	}
	fo.commit(path).map_err(Error::Stdio)
}

fn stackmachine(
	state: &mut Vec<Box<Situation>>,
	out: &mut FileOut,
	buf: &[u8],
	eof: bool,
	sett: &Settings,
) -> Result<usize, Error> {
	let mut pos :usize = 0;
	loop {
		let horizon :&[u8] = &buf[pos .. buf.len()];
		let is_horizon_lengthenable = pos > 0 && !eof;
		let whatnow :WhatNow = try!(state.last_mut().unwrap().as_mut().whatnow(
			&horizon, is_horizon_lengthenable
		).map_err(Error::Syntax));

		if let Some(_) = whatnow.alt {
			out.change = true;
			if sett.osel == OutputSelector::CHECK {
				break;
			}
		}

		try!(out.write_all(&horizon[.. whatnow.pre]).map_err(Error::Stdio));
		let replaceable = &horizon[whatnow.pre .. whatnow.pre + whatnow.len];
		let progress = whatnow.pre + whatnow.len;

		match (whatnow.tri, eof) {
			(Transition::Flush, _) | (Transition::FlushPopOnEof, false) => {
				if progress == 0 {
					break;
				}
			}
			(Transition::Replace(newstate), _) => {
				let ix = state.len() - 1;
				let color_pre;
				let color_final;
				if sett.syntax {
					color_pre = state[ix].get_color();
					color_final = newstate.get_color();
				} else {
					color_pre = COLOR_NORMAL;
					color_final = COLOR_NORMAL;
				};
				try!(write_transition(
					out, sett, replaceable, whatnow.alt,
					color_pre, color_final, color_final,
				).map_err(Error::Stdio));
				state[ix] = newstate;
			}
			(Transition::Push(newstate), _) => {
				let color_pre;
				let color_final;
				if sett.syntax {
					color_pre = state[state.len() - 1].get_color();
					color_final = newstate.get_color();
				} else {
					color_pre = COLOR_NORMAL;
					color_final = COLOR_NORMAL;
				};
				state.push(newstate);
				try!(write_transition(
					out, sett, replaceable, whatnow.alt,
					color_pre, color_final, color_final,
				).map_err(Error::Stdio));
			}
			(Transition::Pop, _) | (Transition::FlushPopOnEof, true) => {
				let color_pre;
				let color_final;
				if sett.syntax {
					color_pre = state[state.len() - 1].get_color();
					color_final = state[state.len() - 2].get_color();
				} else {
					color_pre = COLOR_NORMAL;
					color_final = COLOR_NORMAL;
				};
				state.pop();
				try!(write_transition(
					out, sett, replaceable, whatnow.alt,
					color_pre, color_pre, color_final,
				).map_err(Error::Stdio));
			}
		}
		pos += progress;
	}
	Ok(pos)
}

fn write_transition(
	out: &mut FileOut,
	sett: &Settings,
	replaceable: &[u8],
	alternative: Option<&[u8]>,
	color_pre: u32,
	color_transition: u32,
	color_final: u32,
) -> Result<(), std::io::Error> {
	let mut color_cur = color_pre;
	try!(match (alternative, sett.osel) {
		(Some(replacement), OutputSelector::DIFF) => {
			write_diff(out, &mut color_cur, color_transition, replaceable, &replacement)
		},
		(Some(replacement), OutputSelector::TRANSFORM) => {
			write_colored_slice(out, &mut color_cur, color_transition, replacement)
		},
		(_, _) => {
			write_colored_slice(out, &mut color_cur, color_transition, replaceable)
		},
	});
	if color_cur != color_final {
		try!(write_color(out, color_final));
	}
	Ok(())
}

// Edit distance without replacement; greedy, but that suffices.
fn write_diff(
	out: &mut FileOut,
	mut color_cur: &mut u32,
	color_neutral: u32,
	replaceable: &[u8],
	replacement: &[u8],
) -> Result<(), std::io::Error> {
	let color_a = 0x10800000;
	let color_b = 0x10008000;
	let remain_a = replaceable;
	let mut remain_b = replacement;
	for i in 0 .. remain_a.len() {
		let color_next;
		let a: u8 = remain_a[i];
		if let Some(pivot_b) = remain_b.iter().position(|&b| b == a) {
			color_next = color_neutral;
			try!(write_colored_slice(out, &mut color_cur, color_b, &remain_b[0 .. pivot_b]));
			remain_b = &remain_b[pivot_b+1 ..];
		} else {
			color_next = color_a;
		}
		try!(write_colored_slice(out, &mut color_cur, color_next, &remain_a[i .. i+1]));
	}
	write_colored_slice(out, &mut color_cur, color_b, &remain_b)
}

fn write_colored_slice(
	out: &mut FileOut,
	color_cur: &mut u32,
	color: u32,
	slice: &[u8],
) -> Result<(), std::io::Error> {
	if slice.len() == 0 {
		return Ok(());
	}
	if *color_cur != color {
		try!(write_color(out, color));
		*color_cur = color;
	}
	out.write_all(slice)
}

fn write_color(out :&mut FileOut, code :u32) -> Result<(), std::io::Error> {
	if code == COLOR_NORMAL {
		return out.write_all(b"\x1b[m");
	}

	let bold : &str = if (code >> 24) & 1 == 1 { ";1" } else { "" };
	let ital : &str = if (code >> 25) & 1 == 1 { ";3" } else { "" };

	if code & 0x00ffffff == 0 {
		write!(out, "\x1b[0{}{}m", bold, ital)
	} else {
		let b = code & 0xff;
		let g = (code >> 8) & 0xff;
		let r = (code >> 16) & 0xff;
		let bg = (code >> 28) & 0xf;
		write!(out, "\x1b[0{}{};{}8;2;{};{};{}m", bold, ital, bg+3, r, g, b)
	}
}
