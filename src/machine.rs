/*
 * Copyright 2016 - 2019 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io;
use std::io::Write;

use crate::errfmt::ContextualError;

use crate::filestream::InputSource;
use crate::filestream::FileOut;
use crate::filestream::OutputSink;

use crate::situation::Situation;
use crate::situation::Transition;
use crate::situation::COLOR_NORMAL;

use crate::sitcmd::SitNormal;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
pub enum OutputSelector {
	Original,
	Diff,
	Transform,
	Check,
}

pub struct Settings {
	pub osel :OutputSelector,
	pub syntax :bool,
	pub replace :bool,
}

pub enum Error {
	Stdio(std::io::Error),
	Syntax(ContextualError),
	Check,
}

pub fn treatfile(path: &std::ffi::OsString, sett: &Settings) -> Result<(), Error> {
	let stdin = io::stdin();
	let mut fi: InputSource = if path.is_empty() {
		InputSource::open_stdin(&stdin)
	} else {
		InputSource::open_file(path).map_err(Error::Stdio)?
	};

	let stdout = io::stdout();
	let mut fo: FileOut = if sett.osel == OutputSelector::Check {
		FileOut::open_none()
	} else if sett.replace && !path.is_empty() {
		FileOut::open_soak(fi.size().map_err(Error::Stdio)? * 9 / 8)
	} else {
		FileOut::open_stdout(&stdout)
	};

	let mut color_cur = COLOR_NORMAL;

	let res = treatfile_fallible(&mut fi, &mut fo, &mut color_cur, sett);
	if color_cur != COLOR_NORMAL {
		write_color(&mut fo, COLOR_NORMAL).map_err(Error::Stdio)?;
	}
	if res.is_ok() {
		fo.commit(path).map_err(Error::Stdio)
	} else {
		if let OutputSink::Stdout(mut stdout) = fo.sink {
			let _ = stdout.write_all(b"\n");
		}
		res
	}
}

const MAXHORIZON :usize = 128;

fn treatfile_fallible(
	fi: &mut InputSource, fo: &mut FileOut,
	color_cur: &mut u32, sett: &Settings,
) -> Result<(), Error> {
	let mut fill :usize = 0;
	let mut buf = [0; MAXHORIZON];

	let mut state :Vec<Box<dyn Situation>> = vec!{Box::new(SitNormal {
		end_trigger: 0x100,
		end_replace: None,
	})};

	loop {
		let bytes = fi.read(&mut buf[fill ..]).map_err(Error::Stdio)?;
		fill += bytes;
		let eof = bytes == 0;
		let consumed = stackmachine(
			&mut state, fo, color_cur, &buf[0 .. fill], eof, sett
		)?;
		let remain = fill - consumed;
		if eof {
			assert!(remain == 0);
			break;
		}
		assert!(remain < MAXHORIZON);
		for i in 0 .. remain {
			buf[i] = buf[consumed + i];
		}
		fill = remain;
	}
	if state.len() != 1 {
		return Err(Error::Syntax(ContextualError{
			typ: "Unexpected end of file",
			ctx: buf[0 .. fill].to_owned(),
			pos: fill,
			len: 1,
			msg: "The file's end was reached without closing all sytactic scopes.\n\
			Either, the parser got lost, or the file is truncated or malformed.",
		}));
	}
	Ok(())
}

fn stackmachine(
	state: &mut Vec<Box<dyn Situation>>,
	out: &mut FileOut,
	color_cur: &mut u32,
	buf: &[u8],
	eof: bool,
	sett: &Settings,
) -> Result<usize, Error> {
	let mut pos :usize = 0;
	loop {
		let horizon :&[u8] = &buf[pos ..];
		let is_horizon_lengthenable = horizon.len() < MAXHORIZON && !eof;
		let stacksize_pre = state.len();
		let statebox: &mut Box<dyn Situation> = if let Some(innerstate) = state.last_mut() {
			innerstate
		} else {
			break;
		};
		let curstate = statebox.as_mut();
		let color_pre = if sett.syntax { curstate.get_color() } else { COLOR_NORMAL };
		let whatnow = curstate.whatnow(horizon, is_horizon_lengthenable);
		let (pre, len, alt) = whatnow.transform;

		if alt.is_some() {
			out.change = true;
			if sett.osel == OutputSelector::Check {
				return Err(Error::Check);
			}
		}

		write_colored_slice(
			out, color_cur, color_pre, &horizon[.. pre]
		).map_err(Error::Stdio)?;
		let progress = pre + len;
		let replaceable = &horizon[pre .. progress];

		match (whatnow.transition, eof) {
			(Transition::Flush, _) | (Transition::FlushPopOnEof, false) => {
				if progress == 0 {
					break;
				}
			}
			(Transition::Replace(newstate), _) => {
				*statebox = newstate;
			}
			(Transition::Push(newstate), _) => {
				state.push(newstate);
			}
			(Transition::Pop, _) | (Transition::FlushPopOnEof, true) => {
				state.pop();
			}
			(Transition::Err(e), _) => {
				return Err(Error::Syntax(ContextualError{
					typ: e.typ,
					ctx: buf.to_owned(),
					pos: pos + whatnow.transform.0,
					len: whatnow.transform.1,
					msg: e.msg,
				}));
			}
		}

		let color_trans = if !sett.syntax || state.len() < stacksize_pre {
			color_pre
		} else {
			state.last().unwrap().as_ref().get_color()
		};
		write_transition(
			out, color_cur, color_trans, sett, replaceable, alt
		).map_err(Error::Stdio)?;

		pos += progress;
	}
	Ok(pos)
}

fn write_transition(
	out: &mut FileOut,
	color_cur: &mut u32,
	color_trans: u32,
	sett: &Settings,
	replaceable: &[u8],
	alternative: Option<&[u8]>,
) -> Result<(), std::io::Error> {
	match (alternative, sett.osel) {
		(Some(replacement), OutputSelector::Diff) => {
			write_diff(out, color_cur, color_trans, replaceable, replacement)
		}
		(Some(replacement), OutputSelector::Transform) => {
			write_colored_slice(out, color_cur, color_trans, replacement)
		}
		(_, _) => {
			write_colored_slice(out, color_cur, color_trans, replaceable)
		}
	}?;
	Ok(())
}

// Edit distance without replacement; greedy, but that suffices.
fn write_diff(
	out: &mut FileOut,
	color_cur: &mut u32,
	color_neutral: u32,
	replaceable: &[u8],
	replacement: &[u8],
) -> Result<(), std::io::Error> {
	let color_a = 0x10_800000;
	let color_b = 0x10_008000;
	let remain_a = replaceable;
	let mut remain_b = replacement;
	for (i, &a) in remain_a.iter().enumerate() {
		let color_next;
		if let Some(pivot_b) = remain_b.iter().position(|&b| b == a) {
			color_next = color_neutral;
			write_colored_slice(out, color_cur, color_b, &remain_b[0 .. pivot_b])?;
			remain_b = &remain_b[pivot_b+1 ..];
		} else {
			color_next = color_a;
		}
		write_colored_slice(out, color_cur, color_next, &remain_a[i ..= i])?;
	}
	write_colored_slice(out, color_cur, color_b, remain_b)
}

fn write_colored_slice(
	out: &mut FileOut,
	color_cur: &mut u32,
	color: u32,
	slice: &[u8],
) -> Result<(), std::io::Error> {
	if slice.is_empty() {
		return Ok(());
	}
	if *color_cur != color {
		write_color(out, color)?;
		*color_cur = color;
	}
	out.write_all(slice)
}

fn write_color(out :&mut FileOut, code :u32) -> Result<(), std::io::Error> {
	let zero = if (code >> 24) & 3 != 0 { "0" } else { "" };
	let bold = if (code >> 24) & 1 != 0 { ";1" } else { "" };
	let ital = if (code >> 25) & 1 != 0 { ";3" } else { "" };

	if code & 0x00_ffffff == 0 {
		return write!(out, "\x1b[{}{}{}m", zero, bold, ital);
	}

	let fg = (code >> 28) == 0;
	let b = code & 0xff;
	let g = (code >> 8) & 0xff;
	let r = (code >> 16) & 0xff;
	if fg {
		write!(out, "\x1b[0{}{};38;2;{};{};{}m", bold, ital, r, g, b)
	} else {
		write!(out, "\x1b[0;4{}m", (r >> 7) | (g >> 6) | (b >> 5))
	}
}

pub fn expression_tracker(horizon: &[u8], state: Box<dyn Situation>) -> Result<(bool, usize), ()> {
	let mut stack = vec!{state};
	let mut color_cur = COLOR_NORMAL;

	match stackmachine(
		&mut stack,
		&mut FileOut::open_none(),
		&mut color_cur,
		horizon,
		false,
		&Settings{
			osel: OutputSelector::Original,
			syntax: false,
			replace: false,
		},
	) {
		Ok(len) => Ok((stack.is_empty(), len)),
		Err(_) => Err(()),
	}
}
