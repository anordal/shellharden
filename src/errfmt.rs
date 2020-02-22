/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io::Write;

use crate::syntaxerror::UnsupportedSyntax;

fn stderr_write_or_panic(lock: &mut std::io::StderrLock, bytes: &[u8]) {
	if let Err(e) = lock.write_all(bytes) {
		panic!("Unable to write to stderr: {}", e);
	}
}

pub fn blame_path(path: &std::ffi::OsString, blame: &str) {
	let printable = path.to_string_lossy();
	eprintln!("{}: {}", printable, blame);
}

pub fn blame_path_io(path: &std::ffi::OsString, e: &std::io::Error) {
	let printable = path.to_string_lossy();
	eprintln!("{}: {}", printable, e);
}

pub fn blame_syntax(path: &std::ffi::OsString, fail: &UnsupportedSyntax) {
	blame_path(path, fail.typ);
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

		let stderr = std::io::stderr();
		let mut stderr_lock = stderr.lock();
		stderr_write_or_panic(&mut stderr_lock, &fail.ctx[.. failing_line_end]);
		stderr_write_or_panic(&mut stderr_lock, b"\n");
		for _ in 0 .. width {
			stderr_write_or_panic(&mut stderr_lock, b" ");
		}
		stderr_write_or_panic(&mut stderr_lock, b"^\n");
	}
	eprintln!("{}", fail.msg);
}
