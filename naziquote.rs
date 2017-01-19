use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;

macro_rules! println_stderr(
	($($arg:tt)*) => (
		if let Err(e) = writeln!(&mut io::stderr(), $($arg)* ) {
			panic!("Unable to write to stderr: {}", e);
		}
	)
);

fn perror(blame: std::ffi::OsString, e: &std::io::Error) {
	let printable = blame.to_string_lossy();
	println_stderr!("{}: {}", printable, e);
}

fn main() {
	let mut args: std::env::ArgsOs = env::args_os();
	args.next();

	let mut sett = Settings {
		osel: OutputSelector::DIFF,
		syntax: true,
	};

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
						None
					},
					"--syntax" => {
						sett.osel=OutputSelector::ORIGINAL;
						sett.syntax=true;
						None
					},
					"--syntax-suggest" => {
						sett.osel=OutputSelector::DIFF;
						sett.syntax=true;
						None
					},
					"--besserwisser" => {
						sett.osel=OutputSelector::BESSERWISSER;
						sett.syntax=false;
						None
					},
					"--help" => {
						println!(
							"Naziquote: A bash syntax highlighter that encourages\n\
							(and can fix) proper quoting of variales.\n\
							\n\
							Usage:\n\
							naziquote filename.bash\n\
							cat filename.bash | naziquote ''\n\
							\n\
							Options:\n\
							--suggest         Output a colored diff suggesting changes.\n\
							--syntax          Output syntax highlighting with ANSI colors.\n\
							--syntax-suggest  Diff with syntax highlighting (default mode).\n\
							--besserwisser    Output suggested changes.\n\
							"
						);
						None
					},
					_ => Some(std::ffi::OsString::from(comparable))
				}
			},
			Err(same) => Some(same)
		};
		if let Some(path) = nonopt {
			if let Err(e) = treatfile(&path, &sett) {
				perror(path, &e);
			}
		}
	}
}

#[derive(Clone)]
#[derive(Copy)]
enum OutputSelector {
	ORIGINAL,
	DIFF,
	BESSERWISSER,
}

struct Settings {
	osel :OutputSelector,
	syntax :bool,
}

fn treatfile(path: &std::ffi::OsString, sett: &Settings) -> Result<(), std::io::Error> {
	const BUFSIZE :usize = 128;
	let mut fill :usize = 0;
	let mut buf = [0; BUFSIZE];

	let mut state :Vec<Box<Situation>> = vec!{Box::new(SitCommand{
		end_trigger: b'\x00',
		end_replace: None
	})};

	enum FileOrStdinInput<'a> {
		File(std::fs::File),
		Stdin(std::io::StdinLock<'a>),
	}
	let stdin = io::stdin();
	let mut fh: FileOrStdinInput = if path.is_empty() {
		FileOrStdinInput::Stdin(io::Stdin::lock(&stdin))
	} else {
		FileOrStdinInput::File(try!(File::open(path)))
	};
	let stdout = io::stdout();
	let mut out = stdout.lock();
	loop {
		let bytes = match fh {
			FileOrStdinInput::Stdin(ref mut fh) => try!(fh.read(&mut buf[fill .. BUFSIZE])),
			FileOrStdinInput::File (ref mut fh) => try!(fh.read(&mut buf[fill .. BUFSIZE])),
		};
		fill += bytes;
		let eof = bytes == 0;
		let consumed = try!(stackmachine(&mut state, &mut out, &buf[0 .. fill], eof, &sett));
		let remain = fill - consumed;
		if eof {
			assert!(remain == 0);
			break;
		}
		for i in 0 .. remain {
			buf[i] = buf[consumed + i];
		}
		fill = remain;
	}
	Ok(())
}

fn stackmachine(
	state: &mut Vec<Box<Situation>>,
	out: &mut std::io::StdoutLock,
	buf: &[u8],
	eof: bool,
	sett: &Settings,
) -> Result<usize, std::io::Error> {
	let mut pos :usize = 0;
	while pos < buf.len() {
		let horizon :&[u8] = &buf[pos .. buf.len()];
		let is_horizon_lengthenable = pos > 0 && !eof;
		let whatnow :WhatNow = state.last_mut().unwrap().as_mut().whatnow(
			&horizon, is_horizon_lengthenable
		);

		try!(out.write(&horizon[.. whatnow.pre]));
		let replaceable = &horizon[whatnow.pre .. whatnow.pre + whatnow.len];
		let output_choice :&[u8] = match (whatnow.alt, sett.osel) {
			(Some(replacement), OutputSelector::DIFF) => {
				try!(write_color(out, 0x02800000));
				try!(out.write(replaceable));
				try!(write_color(out, 0x02008000));
				try!(out.write(&replacement));
				try!(write_color(out, COLOR_NORMAL));
				b""
			},
			(Some(replacement), OutputSelector::BESSERWISSER) => replacement,
			(_, _) => replaceable,
		};
		let progress = whatnow.pre + whatnow.len;
		match whatnow.tri {
			Transition::Same => {
				if progress == 0 {
					break;
				}
			}
			Transition::Push(newstate) => {
				state.push(newstate);
				if sett.syntax {
					let color = state.last().unwrap().deref().get_color();
					try!(write_color(out, color));
				}
				try!(out.write(output_choice));
			}
			Transition::Pop => {
				if state.len() > 1 {
					state.pop();
				}
				try!(out.write(output_choice));
				if sett.syntax {
					let color = state.last().unwrap().deref().get_color();
					try!(write_color(out, color));
				}
			}
		}
		pos += progress;
	}
	Ok(pos)
}

const COLOR_NORMAL: u32 = 0xff000000;

fn write_color(out :&mut std::io::StdoutLock, code :u32) -> Result<(), std::io::Error> {
	if code == COLOR_NORMAL {
		write!(out, "\x1b[m")
	} else {
		let b = code & 0xff;
		let g = (code >> 8) & 0xff;
		let r = (code >> 16) & 0xff;
		let bold = (code >> 24) & 0x1;
		let bg = (code >> 25) & 0x7f;
		write!(out, "\x1b[{};{}8;2;{};{};{}m", bold, bg+3, r, g, b)
	}
}

//------------------------------------------------------------------------------

trait Situation {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow;
	fn get_color(&self) -> u32;
}

enum Transition {
	Same,
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

struct SitCommand {
	end_trigger :u8,
	end_replace :Option<&'static [u8]>,
}

impl Situation for SitCommand {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			if horizon[i] == self.end_trigger {
				return WhatNow {
					tri: Transition::Pop, pre: i, len: 1,
					alt: self.end_replace
				};
			}
			if horizon[i] == b'#' {
				return WhatNow {
					tri: Transition::Push(Box::new(SitComment{})),
					pre: i, len: 1, alt: None
				};
			}
			if horizon[i] == b'\'' {
				return WhatNow {
					tri: Transition::Push(Box::new(SitUntilByte{
						until: b'\'', color: 0x00ffff00, end_replace: None
					})),
					pre: i, len: 1, alt: None
				};
			}
			if horizon[i] == b'\"' {
				return WhatNow {
					tri: Transition::Push(Box::new(SitStrDq{})),
					pre: i, len: 1, alt: None
				};
			}
			let common_with_str_dq = common_str_cmd(&horizon, i, is_horizon_lengthenable, true);
			match common_with_str_dq {
				Some(thing) => {
					return thing;
				},
				None => {}
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitStrDq {}

impl Situation for SitStrDq {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\\' {
				let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
				return WhatNow{tri: Transition::Push(esc), pre: i, len: 1, alt: None};
			}
			if horizon[i] == b'\"' {
				return WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None};
			}
			let common_with_cmd = common_str_cmd(&horizon, i, is_horizon_lengthenable, false);
			match common_with_cmd {
				Some(thing) => {
					return thing;
				},
				None => {}
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32{
		0x00ff0000
	}
}

fn flush(i: usize) -> WhatNow {
	WhatNow{tri: Transition::Same, pre: i, len: 0, alt: None}
}

fn common_str_cmd(
	horizon: &[u8],
	i: usize,
	is_horizon_lengthenable: bool,
	need_quotes: bool
) -> Option<WhatNow> {
	if horizon[i] == b'`' {
		let cmd = Box::new(SitCommand{
			end_trigger: b'`',
			end_replace: if_needed(need_quotes, b")\"")
		});
		return Some(WhatNow {
			tri: Transition::Push(cmd),
			pre: i, len: 1, alt: if_needed(need_quotes, b"\"$(")
		});
	}
	if horizon[i] == b'$' {
		if i+1 < horizon.len() {
			let c = horizon[i+1];
			if c == b'\'' {
				return Some(WhatNow {
					tri: Transition::Push(Box::new(SitStrSqEsc{})),
					pre: i, len: 2, alt: None
				});
			} else if c == b'(' {
				let cmd = Box::new(SitCommand{
					end_trigger: b')',
					end_replace: if_needed(need_quotes, b")\"")
				});
				return Some(WhatNow {
					tri: Transition::Push(cmd),
					pre: i, len: 2, alt: if_needed(need_quotes, b"\"$(")
				});
			} else if c == b'#' || c == b'?' {
				let ext = Box::new(SitExtent{
					len: 0,
					color: 0x000000ff,
					end_insert: None
				});
				return Some(WhatNow {
					tri: Transition::Push(ext),
					pre: i, len: 2, alt: None
				});
			} else if c == b'@' || (c >= b'0' && c <= b'9') {
				let ext = Box::new(SitExtent{
					len: 2,
					color: 0x000000ff,
					end_insert: if_needed(need_quotes, b"\"")
				});
				return Some(WhatNow {
					tri: Transition::Push(ext),
					pre: i, len: 0, alt: if_needed(need_quotes, b"\"")
				});
			} else if is_identifierhead(c) {
				let boks = Box::new(SitVarIdent{
					end_replace: if_needed(need_quotes, b"\"")
				});
				return Some(WhatNow {
					tri: Transition::Push(boks),
					pre: i, len: 1, alt: if_needed(need_quotes, b"\"$")
				});
			} else if c == b'{' {
				let cand :&[u8] = &horizon[i+2 ..];
				let idlen = identifierlen(cand);
				let mut rm_braces = false;
				let mut is_number = false;
				if idlen == cand.len() {
					if is_horizon_lengthenable {
						return Some(flush(i));
					}
				} else if cand[idlen] == b'}' {
					if idlen+1 == cand.len() {
						if is_horizon_lengthenable {
							return Some(flush(i));
						}
					} else {
						rm_braces = !is_identifiertail(cand[idlen+1]);
					}
				} else if idlen == 0 && (cand[0] == b'#' || cand[0] == b'?') {
					is_number = true;
				}
				let replace_begin :Option<&'static [u8]>;
				let replace_end   :Option<&'static [u8]>;
				match (need_quotes && !is_number, rm_braces) {
					(true, true) => {
						replace_begin = Some(b"\"$");
						replace_end   = Some(b"\"");
					},
					(true, false) => {
						replace_begin = Some(b"\"${");
						replace_end   = Some(b"}\"");
					},
					(false, true) => {
						replace_begin = Some(b"$");
						replace_end   = Some(b"");
					},
					(false, false) => {
						replace_begin = None;
						replace_end   = None;
					}
				}
				let until = Box::new(SitUntilByte{
					until: b'}', color: 0x000000ff, end_replace: replace_end
				});
				return Some(WhatNow {
					tri: Transition::Push(until),
					pre: i, len: 2, alt: replace_begin
				});
			}
			return Some(flush(i+1));
		}
		return Some(flush(i));
	}
	None
}

fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	return if needed { Some(val) } else { None };
}

struct SitComment {}

impl Situation for SitComment {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\n' {
				return WhatNow{tri: Transition::Pop, pre: 0, len: i, alt: None};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32{
		0x01282828
	}
}

struct SitExtent{
	len : usize,
	color: u32,
	end_insert :Option<&'static [u8]>,
}

impl Situation for SitExtent {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		if horizon.len() >= self.len {
			return WhatNow{tri: Transition::Pop, pre: self.len, len: 0, alt: self.end_insert};
		}
		self.len -= horizon.len();
		return flush(horizon.len());
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
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			if horizon[i] == self.until {
				return WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: self.end_replace};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32{
		self.color
	}
}

struct SitStrSqEsc {}

impl Situation for SitStrSqEsc {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\\' {
				let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
				return WhatNow{tri: Transition::Push(esc), pre: i, len: 1, alt: None};
			}
			if horizon[i] == b'\'' {
				return WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None};
			}
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32{
		0x00ff8000
	}
}

struct SitVarIdent {
	end_replace :Option<&'static [u8]>,
}

impl Situation for SitVarIdent {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> WhatNow {
		let len = identifiertaillen(&horizon);
		if len < horizon.len() {
			return WhatNow{tri: Transition::Pop, pre: len, len: 0, alt: self.end_replace};
		}
		flush(horizon.len())
	}
	fn get_color(&self) -> u32{
		0x000000ff
	}
}

fn identifierlen(horizon: &[u8]) -> usize {
	return if horizon.len() > 0 && is_identifierhead(horizon[0]) {
		1 + identifiertaillen(&horizon[1 ..])
	} else {
		0
	}
}

fn identifiertaillen(horizon: &[u8]) -> usize {
	let mut i: usize = 0;
	while i < horizon.len() && is_identifiertail(horizon[i]) {
		i += 1;
	}
	i
}

fn is_identifierhead(c: u8) -> bool {
	if (c >= b'a' && c <= b'z')
	|| (c >= b'A' && c <= b'Z')
	|| (c == b'_')
	{
		return true;
	}
	return false;
}

fn is_identifiertail(c: u8) -> bool {
	if (c >= b'a' && c <= b'z')
	|| (c >= b'A' && c <= b'Z')
	|| (c >= b'0' && c <= b'9')
	|| (c == b'_')
	{
		return true;
	}
	return false;
}
