/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fmt::{Write as FmtWrite, Arguments};
use std::process;

macro_rules! println_stderr(
	($($arg:tt)*) => (
		if let Err(e) = writeln!(&mut io::stderr(), $($arg)* ) {
			panic!("Unable to write to stderr: {}", e);
		}
	)
);

fn write_bytes_or_panic<L: Write>(locked_io: &mut L, bytes: &[u8]) {
	if let Err(e) = locked_io.write_all(bytes) {
		panic!("Unable to write to stderr: {}", e);
	}
}

fn blame_path(path: std::ffi::OsString, blame: &str) {
	let printable = path.to_string_lossy();
	println_stderr!("{}: {}", printable, blame);
}

fn blame_path_io(path: std::ffi::OsString, e: &std::io::Error) {
	let printable = path.to_string_lossy();
	println_stderr!("{}: {}", printable, e);
}

fn help() {
	println!(
		"Shellharden: A bash syntax highlighter that encourages\n\
		(and can fix) proper quoting of variables.\n\
		\n\
		Usage:\n\
		shellharden filename.bash\n\
		cat filename.bash | shellharden ''\n\
		\n\
		Options:\n\
		--suggest         Output a colored diff suggesting changes.\n\
		--syntax          Output syntax highlighting with ANSI colors.\n\
		--syntax-suggest  Diff with syntax highlighting (default mode).\n\
		--transform       Output suggested changes.\n\
		--check           No output; exit with 2 if changes are suggested.\n\
		--replace         Replace file contents with suggested changes.\n\
		"
	);
}

fn main() {
	let mut args: std::env::ArgsOs = env::args_os();
	args.next();

	let mut sett = Settings {
		osel: OutputSelector::DIFF,
		syntax: true,
		replace: false,
	};

	let mut exit_code: i32 = 0;
	loop {
		let arg = match args.next() {
			Some(arg) => arg,
			None => { break; }
		};
		let nonopt = match arg.into_string() {
			Ok(comparable) => {
				match comparable.as_ref() {
					"--suggest" => {
						sett.osel=OutputSelector::DIFF;
						sett.syntax=false;
						sett.replace=false;
						None
					},
					"--syntax" => {
						sett.osel=OutputSelector::ORIGINAL;
						sett.syntax=true;
						sett.replace=false;
						None
					},
					"--syntax-suggest" => {
						sett.osel=OutputSelector::DIFF;
						sett.syntax=true;
						sett.replace=false;
						None
					},
					"--transform" => {
						sett.osel=OutputSelector::TRANSFORM;
						sett.syntax=false;
						sett.replace=false;
						None
					},
					"--check" => {
						sett.osel=OutputSelector::CHECK;
						sett.syntax=false;
						sett.replace=false;
						None
					},
					"--replace" => {
						sett.osel=OutputSelector::TRANSFORM;
						sett.syntax=false;
						sett.replace=true;
						None
					},
					"--help" => {
						help();
						None
					},
					"-h" => {
						help();
						None
					},
					_ => Some(std::ffi::OsString::from(comparable))
				}
			},
			Err(same) => Some(same)
		};
		if let Some(path) = nonopt {
			if let Err(e) = treatfile(&path, &sett) {
				println!("\x1b[m");
				exit_code = 1;
				match &e {
					&Error::Stdio(ref fail) => blame_path_io(path, &fail),
					&Error::Syntax(ref fail) => {
						blame_path(path, fail.typ);
						blame_syntax(fail);
					},
					&Error::Check => {
						exit_code = 2;
						break;
					},
				};
			}
		}
	}
	process::exit(exit_code);
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
enum OutputSelector {
	ORIGINAL,
	DIFF,
	TRANSFORM,
	CHECK,
}

struct Settings {
	osel :OutputSelector,
	syntax :bool,
	replace :bool,
}

struct UnsupportedSyntax {
	ctx: Vec<u8>,
	pos: usize,
	typ: &'static str,
	msg: &'static str,
}

enum Error {
	Stdio(std::io::Error),
	Syntax(UnsupportedSyntax),
	Check,
}

type ParseResult = Result<WhatNow, UnsupportedSyntax>;

fn blame_syntax(fail: &UnsupportedSyntax) {
	if fail.pos < fail.ctx.len() {
		let mut i = fail.pos;
		while i > 0 {
			i -= 1;
			if fail.ctx[i] == b'\n' {
				break;
			}
		}
		let failing_line_begin = if fail.ctx[i] == b'\n' { i + 1 } else { 0 };
		let mut i = fail.pos;
		while i < fail.ctx.len() && fail.ctx[i] != b'\n' {
			i += 1;
		}
		let failing_line_end = i;
		// FIXME: This counts codepoints, not displayed width.
		let mut width = 0;
		for c in &fail.ctx[failing_line_begin .. fail.pos] {
			if c >> b'\x06' != b'\x02' {
				width += 1;
			}
		}
		let width = width;

		let stderr = io::stderr();
		let mut stderr_lock = stderr.lock();
		write_bytes_or_panic(&mut stderr_lock, &fail.ctx[.. failing_line_end]);
		write_bytes_or_panic(&mut stderr_lock, b"\n");
		for _ in 0 .. width {
			write_bytes_or_panic(&mut stderr_lock, b" ");
		}
		write_bytes_or_panic(&mut stderr_lock, b"^\n");
	}
	println_stderr!("{}", fail.msg);
}

enum InputSource<'a> {
	File(std::fs::File),
	Stdin(std::io::StdinLock<'a>),
}

impl<'a> InputSource<'a> {
	fn open_file(path: &std::ffi::OsString) -> Result<InputSource, std::io::Error> {
		Ok(InputSource::File(try!(File::open(path))))
	}
	fn open_stdin(stdin: &std::io::Stdin) -> InputSource {
		InputSource::Stdin(stdin.lock())
	}
	fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
		match self {
			&mut InputSource::Stdin(ref mut fh) => fh.read(&mut buf),
			&mut InputSource::File (ref mut fh) => fh.read(&mut buf),
		}
	}
	fn size(&mut self) -> Result<u64, std::io::Error> {
		match self {
			&mut InputSource::Stdin(_) => panic!("filesize of stdin"),
			&mut InputSource::File (ref mut fh) => {
				let off :u64 = try!(fh.seek(SeekFrom::End(0)));
				try!(fh.seek(SeekFrom::Start(0)));
				Ok(off)
			},
		}
	}
}

enum OutputSink<'a> {
	Stdout(std::io::StdoutLock<'a>),
	Soak(Vec<u8>),
	None,
}

struct FileOut<'a> {
	sink :OutputSink<'a>,
	change :bool,
}

impl<'a> FileOut<'a> {
	fn open_stdout(stdout: &std::io::Stdout) -> FileOut {
		FileOut{sink: OutputSink::Stdout(stdout.lock()), change: false}
	}
	fn open_soak(reserve: u64) -> FileOut<'a> {
		FileOut{sink: OutputSink::Soak(Vec::with_capacity(reserve as usize)), change: false}
	}
	fn open_none() -> FileOut<'a> {
		FileOut{sink: OutputSink::None, change: false}
	}
	fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
		match &mut self.sink {
			&mut OutputSink::Stdout(ref mut fh) => try!(fh.write_all(&buf)),
			&mut OutputSink::Soak(ref mut vec) => vec.extend_from_slice(buf),
			&mut OutputSink::None => {},
		}
		Ok(())
	}
	fn write_fmt(&mut self, args: Arguments) -> Result<(), std::io::Error> {
		match self.sink {
			OutputSink::Stdout(ref mut fh) => try!(fh.write_fmt(args)),
			OutputSink::Soak(ref mut buf) => {
				// TODO: Format directly to vec<u8>
				let mut s = String::new();
				if let Err(_) = s.write_fmt(args) {
					panic!("fmt::Error");
				}
				buf.extend_from_slice(s.as_bytes());
			},
			OutputSink::None => {},
		}
		Ok(())
	}
	fn commit(&mut self, path: &std::ffi::OsString) -> Result<(), std::io::Error> {
		if self.change {
			match &self.sink {
				&OutputSink::Soak(ref vec) => {
					let mut overwrite = try!(OpenOptions::new().write(true).truncate(true).create(false).open(path));
					try!(overwrite.write_all(vec));
				},
				_ => {},
			}
		}
		Ok(())
	}
}

fn treatfile(path: &std::ffi::OsString, sett: &Settings) -> Result<(), Error> {
	let stdout = io::stdout(); // TODO: not here
	let mut fo: FileOut;
	{
		let stdin = io::stdin(); // TODO: not here
		let mut fi: InputSource = if path.is_empty() {
			InputSource::open_stdin(&stdin)
		} else {
			try!(InputSource::open_file(path).map_err(|e| Error::Stdio(e)))
		};

		fo = if sett.osel == OutputSelector::CHECK {
			FileOut::open_none()
		} else if sett.replace && !path.is_empty() {
			FileOut::open_soak(try!(fi.size().map_err(|e| Error::Stdio(e))) * 9 / 8)
		} else {
			FileOut::open_stdout(&stdout)
		};

		const BUFSIZE :usize = 128;
		let mut fill :usize = 0;
		let mut buf = [0; BUFSIZE];

		let mut state :Vec<Box<Situation>> = vec!{Box::new(SitBeforeFirstArg{
			arg_cmd_data: ArgCmdData{end_trigger: 0x100, end_replace: None},
		})};

		loop {
			let bytes = try!(fi.read(&mut buf[fill ..]).map_err(|e| Error::Stdio(e)));
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
	fo.commit(path).map_err(|e| Error::Stdio(e))
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
		).map_err(|e| Error::Syntax(e)));

		if let Some(_) = whatnow.alt {
			out.change = true;
			if sett.osel == OutputSelector::CHECK {
				break;
			}
		}

		try!(out.write_all(&horizon[.. whatnow.pre]).map_err(|e| Error::Stdio(e)));
		let replaceable = &horizon[whatnow.pre .. whatnow.pre + whatnow.len];
		let progress = whatnow.pre + whatnow.len;
		let whatnow = match whatnow.tri {
			Transition::FlushPopOnEof => {
				if eof {
					WhatNow{
						tri: Transition::Pop,
						pre: whatnow.pre,
						len: whatnow.len,
						alt: whatnow.alt,
					}
				} else {
					WhatNow{
						tri: Transition::Flush,
						pre: whatnow.pre,
						len: 0,
						alt: None,
					}
				}
			},
			_ => whatnow
		};
		match whatnow.tri {
			Transition::Flush => {
				if progress == 0 {
					break;
				}
				if pos == buf.len() {
					break;
				}
			}
			Transition::FlushPopOnEof => {
				panic!("This case shall be filtered out");
			}
			Transition::Replace(newstate) => {
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
					color_pre, color_pre, color_final,
				).map_err(|e| Error::Stdio(e)));
				state[ix] = newstate;
			}
			Transition::Push(newstate) => {
				let color_final = if sett.syntax {
					newstate.get_color()
				} else {
					COLOR_NORMAL
				};
				state.push(newstate);
				try!(write_transition(
					out, sett, replaceable, whatnow.alt,
					COLOR_NORMAL, color_final, color_final,
				).map_err(|e| Error::Stdio(e)));
			}
			Transition::Pop => {
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
				).map_err(|e| Error::Stdio(e)));
			}
		}
		pos += progress;
	}
	Ok(pos)
}

const COLOR_NORMAL: u32 = 0x00000000;
const COLOR_BOLD  : u32 = 0x01000000;

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
	if slice.len() > 0 && *color_cur != color {
		try!(write_color(out, color));
		*color_cur = color;
	}
	try!(out.write_all(slice));
	Ok(())
}

fn write_color(out :&mut FileOut, code :u32) -> Result<(), std::io::Error> {
	let bold = (code >> 24) & 0xf;
	if code & 0x00ffffff == 0 {
		if bold == 0 {
			out.write_all(b"\x1b[m")
		} else {
			write!(out, "\x1b[0;{}m", bold)
		}
	} else {
		let b = code & 0xff;
		let g = (code >> 8) & 0xff;
		let r = (code >> 16) & 0xff;
		let bg = (code >> 28) & 0xf;
		write!(out, "\x1b[{};{}8;2;{};{};{}m", bold, bg+3, r, g, b)
	}
}

//------------------------------------------------------------------------------

trait Situation {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult;
	fn get_color(&self) -> u32;
}

enum Transition {
	Flush,
	FlushPopOnEof,
	Replace(Box<Situation>),
	Push(Box<Situation>),
	Pop,
}

struct WhatNow {
	tri :Transition,
	pre :usize,
	len :usize,
	alt :Option<&'static [u8]>,
}

//------------------------------------------------------------------------------

struct SitBeforeFirstArg {
	arg_cmd_data :ArgCmdData,
}

impl Situation for SitBeforeFirstArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			let a = horizon[i];
			if is_controlcharacter(a) || a == b';' || a == b'|' || a == b'&' {
				continue;
			}
			if a == b'#' {
				return Ok(WhatNow{
					tri: Transition::Push(Box::new(SitUntilByte{
						until: b'\n', color: 0x0320a040, end_replace: None
					})),
					pre: i, len: 1, alt: None
				});
			}
			return Ok(keyword_or_command(
				&self.arg_cmd_data, &horizon, i, is_horizon_lengthenable
			));
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

fn keyword_or_command(
	data :&ArgCmdData,
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
) -> WhatNow {
	let len = predlen(&|x| !is_controlcharacter(x), &horizon[i..]);
	if i + len == horizon.len() && is_horizon_lengthenable {
		return flush(i);
	}
	let word = &horizon[i..i+len];
	if word == b"[[" {
		return WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: vec!{b']', b']'}, color: 0x00007fff}
			)),
			pre: i, len: len, alt: None
		};
	}
	match KEYWORDS_SORTED.binary_search(&word) {
		Ok(_) => WhatNow{
			tri: Transition::Push(Box::new(SitExtent{
				len: len,
				color: 0x01800080,
				end_insert: None
			})), pre: i, len: 0, alt: None
		},
		Err(_) => WhatNow{
			tri: Transition::Replace(Box::new(SitFirstArg{
				arg_cmd_data: data.clone(),
			})), pre: i, len: 0, alt: None
		},
	}
}

static KEYWORDS_SORTED :[&'static[u8]; 13] = [
	b"case",
	b"do",
	b"done",
	b"elif",
	b"else",
	b"esac",
	b"fi",
	b"for",
	b"if",
	b"select",
	b"then",
	b"until",
	b"while",
];

struct SitFirstArg {
	arg_cmd_data :ArgCmdData,
}

impl Situation for SitFirstArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if let Some(res) = common_arg_cmd(&self.arg_cmd_data, horizon, i, is_horizon_lengthenable) {
				return res;
			}
			if is_controlcharacter(horizon[i]) {
				return Ok(WhatNow{
					tri: Transition::Replace(Box::new(SitArg{
						arg_cmd_data: self.arg_cmd_data,
					})), pre: i, len: 1, alt: None
				});
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_BOLD
	}
}

struct SitArg {
	arg_cmd_data :ArgCmdData,
}

impl Situation for SitArg {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if let Some(res) = common_arg_cmd(&self.arg_cmd_data, horizon, i, is_horizon_lengthenable) {
				return res;
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

#[derive(Clone)]
#[derive(Copy)]
struct ArgCmdData {
	end_trigger :u16,
	end_replace :Option<&'static [u8]>,
}

fn common_arg_cmd(
	data :&ArgCmdData,
	horizon :&[u8],
	i :usize,
	is_horizon_lengthenable :bool,
) -> Option<ParseResult> {
	let a = horizon[i];
	if a as u16 == data.end_trigger {
		return Some(Ok(WhatNow{
			tri: Transition::Pop, pre: i, len: 1,
			alt: data.end_replace
		}));
	}
	if a == b'#' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'\n', color: 0x0320a040, end_replace: None
			})),
			pre: i, len: 1, alt: None
		}));
	}
	if a == b'\'' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'\'', color: 0x00ffff00, end_replace: None
			})),
			pre: i, len: 1, alt: None
		}));
	}
	if a == b'\"' {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(SitStrDq{})),
			pre: i, len: 1, alt: None
		}));
	}
	if a == b'\n' || a == b';' || a == b'|' || a == b'&' {
		return Some(Ok(WhatNow{
			tri: Transition::Replace(Box::new(SitBeforeFirstArg{
				arg_cmd_data: data.clone(),
			})), pre: i, len: 0, alt: None
		}));
	}
	match common_str_cmd(&horizon, i, is_horizon_lengthenable, true) {
		CommonStrCmdResult::None => {},
		CommonStrCmdResult::Err(e) => { return Some(Err(e)); },
		CommonStrCmdResult::Ok(consult)
		| CommonStrCmdResult::OnlyWithoutQuotes(consult)=> {
			return Some(Ok(consult));
		},
		CommonStrCmdResult::OnlyWithQuotes(_) => {
			return Some(Ok(WhatNow{
				tri: Transition::Push(Box::new(SitStrPhantom{
					cmd_end_trigger: data.end_trigger,
				})), pre: i, len: 0, alt: Some(b"\"")
			}));
		},
	}
	let (ate, delimiter) = find_heredoc(&horizon[i ..]);
	if i + ate == horizon.len() {
		if is_horizon_lengthenable {
			return Some(Ok(flush(i)));
		}
	} else if delimiter.len() > 0 {
		return Some(Ok(WhatNow{
			tri: Transition::Push(Box::new(
				SitVec{terminator: delimiter, color: 0x0077ff00}
			)),
			pre: i, len: ate, alt: None
		}));
	} else if ate > 0 {
		return Some(Ok(flush(i + ate)));
	}
	None
}

struct SitStrPhantom {
	cmd_end_trigger: u16,
}

impl Situation for SitStrPhantom {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		let mouthful = predlen(&is_phantomstringfood, &horizon);
		if mouthful == horizon.len() {
			if is_horizon_lengthenable {
				return Ok(flush(0));
			}
		} else if horizon[mouthful] as u16 != self.cmd_end_trigger {
			match horizon[mouthful] {
				b'\"' => {
					return Ok(WhatNow{
						tri: Transition::Replace(Box::new(SitStrDq{})),
						pre: mouthful, len: 1, alt: Some(b"")
					});
				}
				b'$' | b'\\' | b'`' => {
					match common_str_cmd(&horizon, mouthful, is_horizon_lengthenable, true) {
						CommonStrCmdResult::None => {},
						CommonStrCmdResult::Err(e) => { return Err(e); },
						CommonStrCmdResult::Ok(consult) |
						CommonStrCmdResult::OnlyWithQuotes(consult) => {
							match &consult.tri {
								&Transition::Flush | &Transition::FlushPopOnEof => {
									return Ok(WhatNow{
										tri: Transition::FlushPopOnEof,
										pre: 0, len: 0, alt: Some(b"\"")
									});
								}
								&Transition::Pop | &Transition::Replace(_) => {}
								&Transition::Push(_) => {
									return Ok(consult);
								}
							}
						},
						CommonStrCmdResult::OnlyWithoutQuotes(_) => {},
					}
				}
				_ => {}
			}
		}
		// Dutifully end the string.
		return Ok(WhatNow{
			tri: Transition::Pop, pre: 0, len: 0, alt: Some(b"\"")
		});
	}
	fn get_color(&self) -> u32{
		0x00ff0000
	}
}

struct SitStrDq {}

impl Situation for SitStrDq {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\"' {
				return Ok(WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None});
			}
			match common_str_cmd(&horizon, i, is_horizon_lengthenable, false) {
				CommonStrCmdResult::None => {},
				CommonStrCmdResult::Err(e) => { return Err(e); },
				CommonStrCmdResult::Ok(x) => { return Ok(x); },
				CommonStrCmdResult::OnlyWithQuotes(x) => { return Ok(x); },
				CommonStrCmdResult::OnlyWithoutQuotes(_) => {
					panic!("Unreachability assertion failed");
				},
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32{
		0x00ff0000
	}
}

fn flush(i: usize) -> WhatNow {
	WhatNow{tri: Transition::Flush, pre: i, len: 0, alt: None}
}

enum CommonStrCmdResult {
	None,
	Err(UnsupportedSyntax),
	Ok(WhatNow),
	OnlyWithQuotes(WhatNow),
	OnlyWithoutQuotes(WhatNow),
}

fn common_str_cmd(
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
	ctx_cmd: bool,
) -> CommonStrCmdResult {
	if horizon[i] == b'`' {
		let cmd = Box::new(SitBeforeFirstArg{
			arg_cmd_data: ArgCmdData{end_trigger: b'`' as u16, end_replace: Some(b")")},
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(cmd), pre: i, len: 1, alt: Some(b"$(")
		});
	}
	if horizon[i] == b'\\' {
		let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
		return CommonStrCmdResult::Ok(WhatNow{
			tri: Transition::Push(esc), pre: i, len: 1, alt: None
		});
	}
	if horizon[i] != b'$' {
		return CommonStrCmdResult::None;
	}
	if i+1 >= horizon.len() {
		if is_horizon_lengthenable {
			return CommonStrCmdResult::Ok(flush(i));
		}
		return CommonStrCmdResult::None;
	}
	let c = horizon[i+1];
	if c == b'\'' {
		if ctx_cmd {
			return CommonStrCmdResult::OnlyWithoutQuotes(WhatNow {
				tri: Transition::Push(Box::new(SitStrSqEsc{})),
				pre: i, len: 2, alt: None
			});
		}
	} else if c == b'(' {
		let cand: &[u8] = &horizon[i+2 ..];
		let (idlen, pos_hazard) = pos_tailhazard(cand, b')');
		if pos_hazard == cand.len() {
			if is_horizon_lengthenable {
				return CommonStrCmdResult::Ok(flush(i));
			}
		} else if idlen == 3 && pos_hazard >= 4 && cand[.. 3].eq(b"pwd") {
			let tailhazard = is_identifiertail(cand[pos_hazard]);
			let replacement: &'static [u8] = if tailhazard {
				b"${PWD}"
			} else {
				b"$PWD"
			};
			let sit = Box::new(SitExtent{
				len: 0,
				color: 0x000000ff,
				end_insert: None,
			});
			return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
				tri: Transition::Push(sit),
				pre: i, len: 6,
				alt: Some(replacement)
			});
		} else if cand.len() >= 1 && cand[0] == b'(' {
			let sit = Box::new(SitVec{
				terminator: vec!{b')', b')'},
				color: 0x00007fff,
			});
			return CommonStrCmdResult::Ok(WhatNow{
				tri: Transition::Push(sit),
				pre: i, len: 3,
				alt: None
			});
		}

		let cmd = Box::new(SitBeforeFirstArg{
			arg_cmd_data: ArgCmdData{end_trigger: b')' as u16, end_replace: None},
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(cmd),
			pre: i, len: 2, alt: None
		});
	} else if c == b'#' || c == b'?' {
		let ext = Box::new(SitExtent{
			len: 2,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::Ok(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 0, alt: None
		});
	} else if c == b'*' {
		let ext = Box::new(SitExtent{
			len: 0,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 2, alt: Some(b"$@")
		});
	} else if predlen(&|c|{c >= b'0' && c <= b'9'}, &horizon[i+1 ..]) > 1 {
		return CommonStrCmdResult::Err(UnsupportedSyntax {
			typ: "Unsuported syntax: Syntactic pitfall",
			ctx: horizon.to_owned(),
			pos: i+2,
			msg: "This does not mean what it looks like. You may be forgiven to think that the full string of \
			numerals is the variable name. Only the fist is.\n\
			\n\
			Try this and be shocked: f() { echo \"$9\" \"$10\"; }; f a b c d e f g h i j\n\
			\n\
			Here is where braces should be used to disambiguate, \
			e.g. \"${10}\" vs \"${1}0\".\n\
			\n\
			Syntactic pitfalls are deemed too dangerous to fix automatically\n\
			(the purpose of Shellharden is to fix brittle code – code that mostly \
			does what it looks like, as opposed to code that never does what it looks like):\n\
			* Fixing what it does would be 100% subtle \
			and might slip through code review unnoticed.\n\
			* Fixing its look would make a likely bug look intentional."
		});
	} else if c == b'@' || (c >= b'0' && c <= b'9') {
		let ext = Box::new(SitExtent{
			len: 2,
			color: 0x000000ff,
			end_insert: None
		});
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(ext),
			pre: i, len: 0, alt: None
		});
	} else if is_identifierhead(c) {
		let tailhazard;
		if ctx_cmd {
			let cand: &[u8] = &horizon[i+1 ..];
			let (_, pos_hazard) = pos_tailhazard(cand, b'\"');
			if pos_hazard == cand.len() {
				if is_horizon_lengthenable {
					return CommonStrCmdResult::Ok(flush(i));
				}
				tailhazard = true;
			} else {
				tailhazard = is_identifiertail(cand[pos_hazard]);
			}
		} else {
			tailhazard = false;
		}
		return CommonStrCmdResult::OnlyWithQuotes(WhatNow{
			tri: Transition::Push(Box::new(SitVarIdent{
				end_insert: if_needed(tailhazard, b"}")
			})), pre: i, len: 1, alt: if_needed(tailhazard, b"${")
		});
	} else if c == b'{' {
		let cand: &[u8] = &horizon[i+2 ..];
		let (idlen, pos_hazard) = pos_tailhazard(cand, b'}');
		let mut rm_braces = false;
		let mut is_number = false;
		if pos_hazard == cand.len() {
			if is_horizon_lengthenable {
				return CommonStrCmdResult::Ok(flush(i));
			}
		} else if idlen < pos_hazard {
			rm_braces = !is_identifiertail(cand[pos_hazard]);
		} else if idlen == 0 && (cand[0] == b'#' || cand[0] == b'?') {
			is_number = true;
		}
		let wn = WhatNow{
			tri: Transition::Push(Box::new(SitUntilByte{
				until: b'}', color: 0x000000ff, end_replace: if_needed(rm_braces, b"")
			})), pre: i, len: 2, alt: if_needed(rm_braces, b"$")
		};
		return if is_number {
			CommonStrCmdResult::Ok(wn)
		} else {
			CommonStrCmdResult::OnlyWithQuotes(wn)
		};
	}
	return CommonStrCmdResult::Ok(flush(i+1));
}

fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	if needed { Some(val) } else { None }
}

struct SitExtent{
	len : usize,
	color: u32,
	end_insert :Option<&'static [u8]>,
}

impl Situation for SitExtent {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		if horizon.len() >= self.len {
			return Ok(WhatNow{tri: Transition::Pop, pre: self.len, len: 0, alt: self.end_insert});
		}
		self.len -= horizon.len();
		return Ok(flush(horizon.len()));
	}
	fn get_color(&self) -> u32{
		self.color
	}
}

struct SitUntilByte {
	until: u8,
	color: u32,
	end_replace :Option<&'static [u8]>,
}

impl Situation for SitUntilByte {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		let len = predlen(&|x| x != self.until, &horizon);
		return Ok(if len < horizon.len() {
			WhatNow{tri: Transition::Pop, pre: len, len: 1, alt: self.end_replace}
		} else {
			WhatNow{
				tri: if is_controlcharacter(self.until) {
					Transition::FlushPopOnEof
				} else {
					Transition::Flush
				}, pre: len, len: 0, alt: None
			}
		});
	}
	fn get_color(&self) -> u32{
		self.color
	}
}

struct SitStrSqEsc {}

impl Situation for SitStrSqEsc {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\\' {
				let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
				return Ok(WhatNow{tri: Transition::Push(esc), pre: i, len: 1, alt: None});
			}
			if horizon[i] == b'\'' {
				return Ok(WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None});
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32{
		0x00ff8000
	}
}

struct SitVarIdent {
	end_insert: Option<&'static [u8]>,
}

impl Situation for SitVarIdent {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		let len = predlen(&is_identifiertail, &horizon);
		if len < horizon.len() {
			return Ok(WhatNow{tri: Transition::Pop, pre: len, len: 0, alt: self.end_insert});
		}
		Ok(WhatNow{
			tri: Transition::FlushPopOnEof,
			pre: horizon.len(), len: 0, alt: self.end_insert
		})
	}
	fn get_color(&self) -> u32{
		0x000000ff
	}
}

struct SitVec {
	terminator :Vec<u8>,
	color: u32,
}

impl Situation for SitVec {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		if horizon.len() < self.terminator.len() {
			if is_horizon_lengthenable {
				Ok(flush(0))
			} else {
				Ok(flush(horizon.len()))
			}
		}
		else if &horizon[0 .. self.terminator.len()] == &self.terminator[..] {
			Ok(WhatNow{tri: Transition::Pop, pre: 0, len: self.terminator.len(), alt: None})
		} else {
			Ok(flush(1))
		}
	}
	fn get_color(&self) -> u32{
		self.color
	}
}

//------------------------------------------------------------------------------

fn pos_tailhazard(horizon: &[u8], end: u8) -> (usize, usize) {
	let idlen = identifierlen(&horizon);
	let mut pos = idlen;
	if idlen < horizon.len() {
		if horizon[pos] == end {
			pos += 1;
			if pos < horizon.len() {
				pos += predlen(&|x| x == b'\"', &horizon[pos ..]);
			}
		}
	}
	return (idlen, pos);
}

fn identifierlen(horizon: &[u8]) -> usize {
	return if horizon.len() > 0 && is_identifierhead(horizon[0]) {
		1 + predlen(&is_identifiertail, &horizon[1 ..])
	} else {
		0
	}
}

fn predlen(pred: &Fn(u8) -> bool, horizon: &[u8]) -> usize {
	let mut i: usize = 0;
	while i < horizon.len() && pred(horizon[i]) {
		i += 1;
	}
	i
}

fn is_identifierhead(c: u8) -> bool {
	(c >= b'a' && c <= b'z')
	|| (c >= b'A' && c <= b'Z')
	|| (c == b'_')
}

fn is_identifiertail(c: u8) -> bool {
	(c >= b'a' && c <= b'z')
	|| (c >= b'A' && c <= b'Z')
	|| (c >= b'0' && c <= b'9')
	|| (c == b'_')
}

fn is_controlcharacter(c: u8) -> bool {
	c <= b' '
}

fn is_phantomstringfood(c: u8) -> bool {
	c >= b'+'
	&& c != b';' && c != b'<' && c != b'>'
	&& c != b'\\' && c != b'`' && c != b'|'
}

fn find_heredoc(horizon: &[u8]) -> (usize, Vec<u8>) {
	let mut ate = predlen(&|x| x == b'<', &horizon);
	let mut found = Vec::<u8>::new();
	if ate != 2 {
		return (ate, found);
	}
	ate += predlen(&|x| x == b'-', &horizon[ate ..]);
	ate += predlen(&is_controlcharacter, &horizon[ate ..]);

	// Lex one word.
	let herein = &horizon[ate ..];
	found.reserve(herein.len());

	#[derive(Clone)]
	#[derive(Copy)]
	enum DelimiterSyntax {
		WORD,
		WORDESC,
		SQ,
		DQ,
		DQESC,
	}
	let mut state = DelimiterSyntax::WORD;

	for byte_ref in herein {
		let byte: u8 = *byte_ref;
		state = match (state, byte) {
			(DelimiterSyntax::WORD, b' ' ) => break,
			(DelimiterSyntax::WORD, b'\n') => break,
			(DelimiterSyntax::WORD, b'\t') => break,
			(DelimiterSyntax::WORD, b'\\') => DelimiterSyntax::WORDESC,
			(DelimiterSyntax::WORD, b'\'') => DelimiterSyntax::SQ,
			(DelimiterSyntax::WORD, b'\"') => DelimiterSyntax::DQ,
			(DelimiterSyntax::SQ, b'\'') => DelimiterSyntax::WORD,
			(DelimiterSyntax::DQ, b'\"') => DelimiterSyntax::WORD,
			(DelimiterSyntax::DQ, b'\\') => DelimiterSyntax::DQESC,
			(DelimiterSyntax::WORDESC, b'\n') => DelimiterSyntax::WORD,
			(DelimiterSyntax::WORDESC, _) => {
				found.push(byte);
				DelimiterSyntax::WORD
			},
			(DelimiterSyntax::DQESC, b'\n') => DelimiterSyntax::DQ,
			(DelimiterSyntax::DQESC, _) => {
				if byte != b'\"' && byte != b'\\' {
					found.push(b'\\');
				}
				found.push(byte);
				DelimiterSyntax::DQ
			},
			(_, _) => {
				found.push(byte);
				state
			},
		};
		ate += 1;
	}
	return (ate, found);
}
