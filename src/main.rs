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
		"Shellharden: The corrective bash syntax highlighter.\n\
		\n\
		Usage:\n\
		shellharden [options] [files]\n\
		cat files | shellharden [options] ''\n\
		\n\
		Options:\n\
		--suggest         Output a colored diff suggesting changes.\n\
		--syntax          Output syntax highlighting with ANSI colors.\n\
		--syntax-suggest  Diff with syntax highlighting (default mode).\n\
		--transform       Output suggested changes.\n\
		--check           No output; exit with 2 if changes are suggested.\n\
		--replace         Replace file contents with suggested changes.\n\
		--                Don't treat further arguments as options.\n\
		-h|--help         Show help text.\n\
		--version         Show version.\n\
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
	let mut opt_trigger: &str = "-";
	while let Some(arg) = args.next() {
		if let Some(comparable) = arg.to_str() {
			if comparable.starts_with(opt_trigger) {
				match comparable {
					"--suggest" => {
						sett.osel=OutputSelector::DIFF;
						sett.syntax=false;
						sett.replace=false;
						continue;
					},
					"--syntax" => {
						sett.osel=OutputSelector::ORIGINAL;
						sett.syntax=true;
						sett.replace=false;
						continue;
					},
					"--syntax-suggest" => {
						sett.osel=OutputSelector::DIFF;
						sett.syntax=true;
						sett.replace=false;
						continue;
					},
					"--transform" => {
						sett.osel=OutputSelector::TRANSFORM;
						sett.syntax=false;
						sett.replace=false;
						continue;
					},
					"--check" => {
						sett.osel=OutputSelector::CHECK;
						sett.syntax=false;
						sett.replace=false;
						continue;
					},
					"--replace" => {
						sett.osel=OutputSelector::TRANSFORM;
						sett.syntax=false;
						sett.replace=true;
						continue;
					},
					"--help" | "-h" => {
						help();
						continue;
					},
					"--version" => {
						println!(env!("CARGO_PKG_VERSION"));
						continue;
					},
					"--" => {
						opt_trigger = "\x00";
						continue;
					},
					_ => {
						errfmt::blame_path(&arg, "No such option.");
						exit_code = 3;
						break;
					},
				}
			}
		}
		if let Err(e) = machine::treatfile(&arg, &sett) {
			println!("\x1b[m");
			exit_code = 1;
			match &e {
				&machine::Error::Stdio(ref fail) => {
					errfmt::blame_path_io(&arg, &fail);
				},
				&machine::Error::Syntax(ref fail) => {
					errfmt::blame_syntax(&arg, &fail);
				},
				&machine::Error::Check => {
					exit_code = 2;
					break;
				},
			};
		}
	}
	process::exit(exit_code);
}

//------------------------------------------------------------------------------

mod commonargcmd;
mod commonstrcmd;
mod microparsers;
mod sitcase;
mod sitcomment;
mod sitcmd;
mod sitrvalue;
mod sitvec;
mod sitvarident;
mod sitstrsqesc;
mod situntilbyte;
mod sitextent;
mod sitstrdq;
mod sitstrphantom;
