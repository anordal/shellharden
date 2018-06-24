/*
 * Copyright 2016 - 2018 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::env;
use std::process;

mod machine;
use machine::OutputSelector;

mod errfmt;
mod filestream;
mod situation;
mod syntaxerror;

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

	let mut sett = machine::Settings {
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
			if let Err(e) = machine::treatfile(&path, &sett) {
				println!("\x1b[m");
				exit_code = 1;
				match &e {
					&machine::Error::Stdio(ref fail) => {
						errfmt::blame_path_io(&path, &fail);
					},
					&machine::Error::Syntax(ref fail) => {
						errfmt::blame_syntax(&path, &fail);
					},
					&machine::Error::Check => {
						exit_code = 2;
						break;
					},
				};
			}
		}
	}
	process::exit(exit_code);
}

//------------------------------------------------------------------------------

mod commonstrcmd;
mod microparsers;
mod sitcomment;
mod sitcmd;
mod sitvec;
mod sitvarident;
mod sitstrsqesc;
mod situntilbyte;
mod sitextent;
mod sitstrdq;
mod sitstrphantom;
