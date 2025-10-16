/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io::{Read, Seek, Write};
use std::fmt::{Write as FmtWrite};

pub enum InputSource<'a> {
	File(std::fs::File),
	Stdin(std::io::StdinLock<'a>),
}

impl<'a> InputSource<'a> {
	pub fn open_file(path: &std::ffi::OsString) -> Result<InputSource<'a>, std::io::Error> {
		Ok(InputSource::File(std::fs::File::open(path)?))
	}
	pub fn open_stdin(stdin: &std::io::Stdin) -> InputSource<'a> {
		InputSource::Stdin(stdin.lock())
	}
	pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
		match *self {
			InputSource::Stdin(ref mut fh) => fh.read(buf),
			InputSource::File (ref mut fh) => fh.read(buf),
		}
	}
	pub fn size(&mut self) -> Result<u64, std::io::Error> {
		match *self {
			InputSource::Stdin(_) => panic!("filesize of stdin"),
			InputSource::File (ref mut fh) => {
				let off :u64 = fh.seek(std::io::SeekFrom::End(0))?;
				fh.seek(std::io::SeekFrom::Start(0))?;
				Ok(off)
			}
		}
	}
}

pub enum OutputSink<'a> {
	Stdout(std::io::StdoutLock<'a>),
	Soak(Vec<u8>),
	None,
}

pub struct FileOut<'a> {
	pub sink :OutputSink<'a>,
	pub change :bool,
}

impl<'a> FileOut<'a> {
	pub fn open_stdout(stdout: &std::io::Stdout) -> FileOut<'a> {
		FileOut{sink: OutputSink::Stdout(stdout.lock()), change: false}
	}
	pub fn open_soak(reserve: u64) -> FileOut<'a> {
		FileOut{sink: OutputSink::Soak(Vec::with_capacity(reserve as usize)), change: false}
	}
	pub fn open_none() -> FileOut<'a> {
		FileOut{sink: OutputSink::None, change: false}
	}
	pub fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
		match self.sink {
			OutputSink::Stdout(ref mut fh) => fh.write_all(buf)?,
			OutputSink::Soak(ref mut vec) => vec.extend_from_slice(buf),
			OutputSink::None => {}
		}
		Ok(())
	}
	pub fn write_fmt(&mut self, args: std::fmt::Arguments) -> Result<(), std::io::Error> {
		match self.sink {
			OutputSink::Stdout(ref mut fh) => fh.write_fmt(args)?,
			OutputSink::Soak(ref mut buf) => {
				// TODO: Format directly to vec<u8>
				let mut s = String::new();
				if s.write_fmt(args).is_err() {
					panic!("fmt::Error");
				}
				buf.extend_from_slice(s.as_bytes());
			}
			OutputSink::None => {}
		}
		Ok(())
	}
	pub fn commit(&mut self, path: &std::ffi::OsString) -> Result<(), std::io::Error> {
		if self.change {
			if let OutputSink::Soak(ref vec) = self.sink {
				std::fs::OpenOptions::new()
					.write(true)
					.truncate(true)
					.create(false)
					.open(path)?
					.write_all(vec)?
				;
			}
		}
		Ok(())
	}
}
