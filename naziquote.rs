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
	let args: Vec<String> = env::args().collect();

	for arg in &args[1..] {
		match treatfile(arg) {
			Ok(()) => {},
			Err(e) => {
				println!("{}: {}", arg, e);
			},
		}
	}
}

fn treatfile(path: &str) -> Result<(), std::io::Error> {
	const BUFSIZE :usize = 128;
	let mut fill :usize = 0;
	let mut buf = [0; BUFSIZE];

	let mut state :Vec<Box<Situation>> = vec!{Box::new(SitCommand{})};

	let mut fh = try!(File::open(path));
	let stdout = io::stdout();
	let mut out = stdout.lock();
	loop {
		let bytes = try!(fh.read(&mut buf[fill .. BUFSIZE]));
		fill += bytes;
		let eof = bytes == 0;
		let consumed = try!(stackmachine(&mut state, &mut out, &buf[0 .. fill], eof));
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
	state :&mut Vec<Box<Situation>>,
	out :&mut std::io::StdoutLock,
	buf :&[u8],
	eof :bool
) -> Result<usize, std::io::Error> {
	let mut pos :usize = 0;
	while pos < buf.len() {
		let horizon :&[u8] = &buf[pos .. buf.len()];
		let whatnow :WhatNow = state.last().unwrap().deref().whatnow(&horizon, eof);

		match whatnow.tr {
			Transition::Same => {}
			Transition::Swap(newstate) => {
				*state.last_mut().unwrap() = newstate;
			}
			Transition::Push(newstate) => {
				state.push(newstate);
			}
			Transition::Pop => {
				if state.len() > 1 {
					state.pop();
				}
			}
		}
		let output :&[u8] = match whatnow.repl {
			Some(replacement) => replacement,
			None => &horizon[.. whatnow.took],
		};
		try!(out.write(&output));
		try!(write_color(out, state.last().unwrap().deref().get_color()));
		pos += whatnow.took;
	}
	Ok(pos)
}

fn write_color(out :&mut std::io::StdoutLock, code :u32) -> Result<(), std::io::Error> {
	let b = code & 0xff;
	let g = (code >> 8) & 0xff;
	let r = (code >> 16) & 0xff;
	let ansi = (code >> 24) & 0xff;
	write!(out, "\x1b[{};38;2;{};{};{}m", ansi, r, g, b)
}

//------------------------------------------------------------------------------

trait Situation {
	fn whatnow(&self, horizon: &[u8], eof: bool) -> WhatNow;
	fn get_color(&self) -> u32;
}

enum Transition {
	Same,
	Swap(Box<Situation>),
	Push(Box<Situation>),
	Pop,
}

struct WhatNow {
	tr :Transition,
	took :usize,
	repl :Option<&'static [u8]>,
}

//------------------------------------------------------------------------------

struct SitCommand {}

impl Situation for SitCommand {
	fn whatnow(&self, horizon: &[u8], eof: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			let c = horizon[i];
			if c == b'#' {
				return WhatNow{tr: Transition::Push(Box::new(SitComment{})), took: i, repl: None};
			}
		}
		return WhatNow{tr: Transition::Same, took: horizon.len(), repl: None};
	}
	fn get_color(&self) -> u32 {
		0x00a0a0a0
	}
}

struct SitComment {}

impl Situation for SitComment {
	fn whatnow(&self, horizon: &[u8], eof: bool) -> WhatNow {
		for i in 0 .. horizon.len() {
			let c = horizon[i];
			if c == b'\n' {
				return WhatNow{tr: Transition::Pop, took: i+1, repl: None};
			}
		}
		return WhatNow{tr: Transition::Same, took: horizon.len(), repl: None};
	}
	fn get_color(&self) -> u32{
		0x01282828
	}
}
