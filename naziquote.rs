use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;

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
	const LOOKAHEAD :usize = 80;
	const SLACK     :usize = 48;
	const BUFSIZE   :usize = SLACK + LOOKAHEAD;
	let mut fill    :usize = 0;
	let mut buf = [0; BUFSIZE];

	let mut fh = try!(File::open(path));
	let stdout = io::stdout();
	let mut out = stdout.lock();
	loop {
		let bytes = try!(fh.read(&mut buf[fill .. BUFSIZE]));
		fill += bytes;
		if fill <= LOOKAHEAD {
			if bytes != 0 {
				continue;
			}
			break;
		}
		let usable = fill - LOOKAHEAD;
		try!(stackmachine(&mut out, &buf[0 .. fill], usable));
		for i in 0 .. LOOKAHEAD {
			buf[i] = buf[usable + i];
		}
		fill = LOOKAHEAD;
	}
	try!(stackmachine(&mut out, &buf[0 .. fill], fill));
	Ok(())
}

fn stackmachine(out :&mut std::io::StdoutLock, buf :&[u8], usable :usize) -> Result<(), std::io::Error> {
	try!(out.write(&buf[0 .. usable]));
	Ok(())
}
