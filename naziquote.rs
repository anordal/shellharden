use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
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

fn main() {
	let mut args: std::env::ArgsOs = env::args_os();
	args.next();

	let mut sett = Settings {
		osel: OutputSelector::DIFF,
		syntax: true,
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
				println!("\x1b[m");
				perror_error(path, &e);
				exit_code = 1;
			}
		}
	}
	process::exit(exit_code);
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

struct UnsupportedSyntax {
	ctx: Vec<u8>,
	pos: usize,
	typ: &'static str,
	msg: &'static str,
}

enum Error {
	Stdio(std::io::Error),
	Syntax(UnsupportedSyntax),
}

type ParseResult = Result<WhatNow, UnsupportedSyntax>;

fn perror_error(path: std::ffi::OsString, e: &Error) {
	match e {
		&Error::Stdio(ref fail) => { blame_path_io(path, &fail); },
		&Error::Syntax(ref fail) => {
			blame_path(path, fail.typ);
			blame_syntax(fail);
		},
	}
}

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

enum FileOrStdinInput<'a> {
	File(std::fs::File),
	Stdin(std::io::StdinLock<'a>),
}

trait OpenAndRead {
	fn open_file(path: &std::ffi::OsString) -> Result<FileOrStdinInput, std::io::Error>;
	fn open_stdin(stdin: &std::io::Stdin) -> FileOrStdinInput;
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error>;
}

impl<'a> OpenAndRead for FileOrStdinInput<'a> {
	fn open_file(path: &std::ffi::OsString) -> Result<FileOrStdinInput, std::io::Error> {
		Ok(FileOrStdinInput::File(try!(File::open(path))))
	}
	fn open_stdin(stdin: &std::io::Stdin) -> FileOrStdinInput {
		FileOrStdinInput::Stdin(io::Stdin::lock(&stdin))
	}
	fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
		match self {
			&mut FileOrStdinInput::Stdin(ref mut fh) => fh.read(&mut buf),
			&mut FileOrStdinInput::File (ref mut fh) => fh.read(&mut buf),
		}
	}
}

fn treatfile(path: &std::ffi::OsString, sett: &Settings) -> Result<(), Error> {
	const BUFSIZE :usize = 128;
	let mut fill :usize = 0;
	let mut buf = [0; BUFSIZE];

	let mut state :Vec<Box<Situation>> = vec!{Box::new(SitCommand{
		end_trigger: 0x100,
		end_replace: None
	})};

	let stdin = io::stdin();
	let mut fh: FileOrStdinInput = if path.is_empty() {
		FileOrStdinInput::open_stdin(&stdin)
	} else {
		try!(FileOrStdinInput::open_file(path).map_err(|e| Error::Stdio(e)))
	};
	let stdout = io::stdout();
	let mut out = stdout.lock();
	loop {
		let bytes = try!(fh.read(&mut buf[fill ..]).map_err(|e| Error::Stdio(e)));
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
	if state.len() == 1 {
		Ok(())
	} else {
		Err(Error::Syntax(UnsupportedSyntax{
			typ: "Unexpected end of file",
			ctx: buf[0 .. fill].to_owned(),
			pos: fill,
			msg: "The file's end was reached without closing all sytactic scopes.\n\
			Either, the parser got lost, or the file is truncated or malformed.",
		}))
	}
}

fn stackmachine(
	state: &mut Vec<Box<Situation>>,
	out: &mut std::io::StdoutLock,
	buf: &[u8],
	eof: bool,
	sett: &Settings,
) -> Result<usize, Error> {
	let mut pos :usize = 0;
	while pos < buf.len() {
		let horizon :&[u8] = &buf[pos .. buf.len()];
		let is_horizon_lengthenable = pos > 0 && !eof;
		let whatnow :WhatNow = try!(state.last_mut().unwrap().as_mut().whatnow(
			&horizon, is_horizon_lengthenable
		).map_err(|e| Error::Syntax(e)));

		try!(out.write(&horizon[.. whatnow.pre]).map_err(|e| Error::Stdio(e)));
		let replaceable = &horizon[whatnow.pre .. whatnow.pre + whatnow.len];
		let progress = whatnow.pre + whatnow.len;
		match whatnow.tri {
			Transition::Same => {
				if progress == 0 {
					break;
				}
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

const COLOR_NORMAL: u32 = 0xff000000;

fn write_transition(
	out: &mut std::io::StdoutLock,
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
		(Some(replacement), OutputSelector::BESSERWISSER) => {
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
	out: &mut std::io::StdoutLock,
	mut color_cur: &mut u32,
	color_neutral: u32,
	replaceable: &[u8],
	replacement: &[u8],
) -> Result<(), std::io::Error> {
	let color_a = 0x02800000;
	let color_b = 0x02008000;
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
	out: &mut std::io::StdoutLock,
	mut color_cur: &mut u32,
	color: u32,
	slice: &[u8],
) -> Result<(), std::io::Error> {
	if slice.len() > 0 && *color_cur != color {
		try!(write_color(out, color));
		*color_cur = color;
	}
	try!(out.write(slice));
	Ok(())
}

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
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult;
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
	end_trigger :u16,
	end_replace :Option<&'static [u8]>,
}

impl Situation for SitCommand {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] as u16 == self.end_trigger {
				return Ok(WhatNow {
					tri: Transition::Pop, pre: i, len: 1,
					alt: self.end_replace
				});
			}
			if horizon[i] == b'#' {
				return Ok(WhatNow {
					tri: Transition::Push(Box::new(SitComment{})),
					pre: i, len: 1, alt: None
				});
			}
			if horizon[i] == b'\'' {
				return Ok(WhatNow {
					tri: Transition::Push(Box::new(SitUntilByte{
						until: b'\'', color: 0x00ffff00, end_replace: None
					})),
					pre: i, len: 1, alt: None
				});
			}
			if horizon[i] == b'\"' {
				return Ok(WhatNow {
					tri: Transition::Push(Box::new(SitStrDq{})),
					pre: i, len: 1, alt: None
				});
			}
			let common_with_str_dq = try!(common_str_cmd(&horizon, i, is_horizon_lengthenable, true));
			match common_with_str_dq {
				Some(thing) => {
					return Ok(thing);
				},
				None => {}
			}
			let (ate, delimiter) = find_heredoc(&horizon[i ..]);
			if i + ate == horizon.len() {
				if is_horizon_lengthenable {
					return Ok(flush(i));
				}
			} else if delimiter.len() > 0 {
				return Ok(WhatNow{
					tri: Transition::Push(Box::new(
						SitHeredoc{terminator: delimiter}
					)),
					pre: i, len: ate, alt: None
				});
			} else if ate > 0 {
				return Ok(flush(i + ate));
			}
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32 {
		COLOR_NORMAL
	}
}

struct SitStrDq {}

impl Situation for SitStrDq {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\"' {
				return Ok(WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: None});
			}
			let common_with_cmd = try!(common_str_cmd(&horizon, i, is_horizon_lengthenable, false));
			match common_with_cmd {
				Some(thing) => {
					return Ok(thing);
				},
				None => {}
			}
		}
		Ok(flush(horizon.len()))
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
) -> Result<Option<WhatNow>, UnsupportedSyntax> {
	if horizon[i] == b'`' {
		let cmd = Box::new(SitCommand{
			end_trigger: b'`' as u16,
			end_replace: if_needed(need_quotes, b")\"")
		});
		return Ok(Some(WhatNow {
			tri: Transition::Push(cmd),
			pre: i, len: 1, alt: if_needed(need_quotes, b"\"$(")
		}));
	}
	if horizon[i] == b'\\' {
		let esc = Box::new(SitExtent{len: 1, color: 0x01ff0080, end_insert: None});
		return Ok(Some(WhatNow{
			tri: Transition::Push(esc), pre: i, len: 1, alt: None
		}));
	}
	if horizon[i] == b'$' {
		if i+1 < horizon.len() {
			let c = horizon[i+1];
			if c == b'\'' {
				if need_quotes {
					return Ok(Some(WhatNow {
						tri: Transition::Push(Box::new(SitStrSqEsc{})),
						pre: i, len: 2, alt: None
					}));
				}
			} else if c == b'(' {
				let cmd_end = identifierlen(&horizon[i+2 ..]);
				if i+2+cmd_end+1 >= horizon.len() {
					if is_horizon_lengthenable {
						return Ok(Some(flush(i+1)));
					}
				} else if horizon[i+2+cmd_end] == b')' && horizon[i+2 .. i+2+cmd_end].eq(b"pwd") {
					let replacement: &'static [u8] = if need_quotes {
						b"\"$PWD\""
					} else {
						let tailhazard = is_identifiertail(horizon[i+2+cmd_end+1]);
						if tailhazard {
							b"${PWD}"
						} else {
							b"$PWD"
						}
					};
					let sit = Box::new(SitExtent{
						len: 0,
						color: 0x000000ff,
						end_insert: None,
					});
					return Ok(Some(WhatNow {
						tri: Transition::Push(sit),
						pre: i, len: 6,
						alt: Some(replacement)
					}));
				}

				let cmd = Box::new(SitCommand{
					end_trigger: b')' as u16,
					end_replace: if_needed(need_quotes, b")\"")
				});
				return Ok(Some(WhatNow {
					tri: Transition::Push(cmd),
					pre: i, len: 2, alt: if_needed(need_quotes, b"\"$(")
				}));
			} else if c == b'#' || c == b'?' {
				let ext = Box::new(SitExtent{
					len: 0,
					color: 0x000000ff,
					end_insert: None
				});
				return Ok(Some(WhatNow {
					tri: Transition::Push(ext),
					pre: i, len: 2, alt: None
				}));
			} else if c == b'*' {
				let ext = Box::new(SitExtent{
					len: 0,
					color: 0x000000ff,
					end_insert: None
				});
				return Ok(Some(WhatNow {
					tri: Transition::Push(ext),
					pre: i, len: 2, alt: if need_quotes { Some(b"\"$@\"") } else { Some(b"$@") }
				}));
			} else if predlen(&|c|{c >= b'0' && c <= b'9'}, &horizon[i+1 ..]) > 1 {
				return Err(UnsupportedSyntax {
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
					(the purpose of Naziquote is to fix brittle code – code that mostly \
					does what it looks like, as opposed to code that never does what it looks like):\n\
					* Fixing what it does would be 100% subtle \
					and might slip through code review unnoticed.\n\
					* Fixing its look would make a likely bug look intentional."
				});
			} else if c == b'@' || (c >= b'0' && c <= b'9') {
				let ext = Box::new(SitExtent{
					len: 2,
					color: 0x000000ff,
					end_insert: if_needed(need_quotes, b"\"")
				});
				return Ok(Some(WhatNow {
					tri: Transition::Push(ext),
					pre: i, len: 0, alt: if_needed(need_quotes, b"\"")
				}));
			} else if is_identifierhead(c) {
				let boks = Box::new(SitVarIdent{
					end_replace: if_needed(need_quotes, b"\"")
				});
				return Ok(Some(WhatNow {
					tri: Transition::Push(boks),
					pre: i, len: 1, alt: if_needed(need_quotes, b"\"$")
				}));
			} else if c == b'{' {
				let cand :&[u8] = &horizon[i+2 ..];
				let idlen = identifierlen(cand);
				let mut rm_braces = false;
				let mut is_number = false;
				if idlen == cand.len() {
					if is_horizon_lengthenable {
						return Ok(Some(flush(i)));
					}
				} else if cand[idlen] == b'}' {
					if idlen+1 == cand.len() {
						if is_horizon_lengthenable {
							return Ok(Some(flush(i)));
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
				return Ok(Some(WhatNow {
					tri: Transition::Push(until),
					pre: i, len: 2, alt: replace_begin
				}));
			}
			return Ok(Some(flush(i+1)));
		}
		return Ok(Some(flush(i)));
	}
	Ok(None)
}

fn if_needed<T>(needed: bool, val: T) -> Option<T> {
	return if needed { Some(val) } else { None };
}

struct SitComment {}

impl Situation for SitComment {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		for i in 0 .. horizon.len() {
			if horizon[i] == b'\n' {
				return Ok(WhatNow{tri: Transition::Pop, pre: 0, len: i, alt: None});
			}
		}
		Ok(flush(horizon.len()))
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
		for i in 0 .. horizon.len() {
			if horizon[i] == self.until {
				return Ok(WhatNow{tri: Transition::Pop, pre: i, len: 1, alt: self.end_replace});
			}
		}
		Ok(flush(horizon.len()))
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
	end_replace :Option<&'static [u8]>,
}

impl Situation for SitVarIdent {
	#[allow(unused_variables)]
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		let len = predlen(&is_identifiertail, &horizon);
		if len < horizon.len() {
			return Ok(WhatNow{tri: Transition::Pop, pre: len, len: 0, alt: self.end_replace});
		}
		Ok(flush(horizon.len()))
	}
	fn get_color(&self) -> u32{
		0x000000ff
	}
}

struct SitHeredoc {
	terminator :Vec<u8>,
}

impl Situation for SitHeredoc {
	fn whatnow(&mut self, horizon: &[u8], is_horizon_lengthenable: bool) -> ParseResult {
		if horizon.len() < self.terminator.len() {
			if is_horizon_lengthenable {
				return Ok(flush(0));
			}
		}
		else if &horizon[0 .. self.terminator.len()] == &self.terminator[..] {
			return Ok(WhatNow{tri: Transition::Pop, pre: 0, len: self.terminator.len(), alt: None});
		}
		return Ok(flush(1));
	}
	fn get_color(&self) -> u32{
		0x0077ff00
	}
}

//------------------------------------------------------------------------------

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

fn is_controlcharacter(c: u8) -> bool {
	return c <= b' ';
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
