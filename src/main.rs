/*
 * Copyright 2016 - 2021 Andreas Nordal
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// Color codes are split between flag- and color bits on purpose, such as 0x03_789060.
#![allow(clippy::unusual_byte_groupings)]

use std::env;
use std::process;
use std::ffi::OsStr;

mod machine;
use crate::machine::OutputSelector;

mod errfmt;
mod filestream;
mod situation;

fn help() {
	println!(
		"Shellharden: The corrective bash syntax highlighter.\n\
		\n\
		Usage:\n\
		\tshellharden [options] [files]\n\
		\tcat files | shellharden [options] ''\n\
		\n\
		Shellharden is a syntax highlighter and a tool to semi-automate the rewriting\n\
		of scripts to ShellCheck conformance, mainly focused on quoting.\n\
		\n\
		The default mode of operation is like `cat`, but with syntax highlighting in\n\
		foreground colors and suggestive changes in background colors.\n\
		\n\
		Options:\n\
		\t--suggest         Output a colored diff suggesting changes.\n\
		\t--syntax          Output syntax highlighting with ANSI colors.\n\
		\t--syntax-suggest  Diff with syntax highlighting (default mode).\n\
		\t--transform       Output suggested changes.\n\
		\t--check           No output; exit with 2 if changes are suggested.\n\
		\t--replace         Replace file contents with suggested changes.\n\
		\t--                Don't treat further arguments as options.\n\
		\t-h|--help         Show help text.\n\
		\t--version         Show version.\n\
		\n\
		The changes suggested by Shellharden inhibits word splitting and indirect\n\
		pathname expansion. This will make your script ShellCheck compliant in terms of\n\
		quoting. Whether your script will work afterwards is a different question:\n\
		If your script was using those features on purpose, it obviously won't anymore!\n\
		\n\
		Every script is possible to write without using word splitting or indirect\n\
		pathname expansion, but it may involve doing things differently.\n\
		See the accompanying file how_to_do_things_safely_in_bash.md or online:\n\
		https://github.com/anordal/shellharden/blob/master/how_to_do_things_safely_in_bash.md\n\
		"
	);
}

fn main() {
	let mut args: std::env::ArgsOs = env::args_os();
	args.next();

	let mut sett = machine::Settings {
		osel: OutputSelector::Diff,
		syntax: true,
		replace: false,
	};

	let mut exit_code: i32 = 0;
	let mut opt_trigger: &str = "-";
	for arg in args {
		if let Some(option) = get_if_opt(&arg, opt_trigger) {
			match option {
				"--suggest" => {
					sett.osel = OutputSelector::Diff;
					sett.syntax = false;
					sett.replace = false;
				}
				"--syntax" => {
					sett.osel = OutputSelector::Original;
					sett.syntax = true;
					sett.replace = false;
				}
				"--syntax-suggest" => {
					sett.osel = OutputSelector::Diff;
					sett.syntax = true;
					sett.replace = false;
				}
				"--transform" => {
					sett.osel = OutputSelector::Transform;
					sett.syntax = false;
					sett.replace = false;
				}
				"--check" => {
					sett.osel = OutputSelector::Check;
					sett.syntax = false;
					sett.replace = false;
				}
				"--replace" => {
					sett.osel = OutputSelector::Transform;
					sett.syntax = false;
					sett.replace = true;
				}
				"--help" | "-h" => {
					help();
				}
				"--version" => {
					println!(env!("CARGO_PKG_VERSION"));
				}
				"--" => {
					opt_trigger = "\x00";
				}
				_ => {
					errfmt::blame_path(&arg, "No such option.");
					exit_code = 3;
					break;
				}
			}
		}
		else if let Err(e) = machine::treatfile(&arg, &sett) {
			exit_code = 1;
			match (sett.osel, e) {
				(_, machine::Error::Stdio(ref fail)) => {
					errfmt::blame_path_io(&arg, fail);
				}
				(OutputSelector::Check, _) | (_, machine::Error::Check) => {
					exit_code = 2;
					break;
				}
				(_, machine::Error::Syntax(ref fail)) => {
					errfmt::blame_syntax(&arg, fail);
				}
			};
		}
	}
	process::exit(exit_code);
}

fn get_if_opt<'a>(arg: &'a OsStr, opt_trigger: &str) -> Option<&'a str> {
	if let Some(comparable) = arg.to_str() {
		if comparable.starts_with(opt_trigger) {
			return Some(comparable);
		}
	}
	None
}

//------------------------------------------------------------------------------

#[cfg(test)]
#[macro_use]
mod testhelpers;

mod commonargcmd;
mod commonstrcmd;
mod microparsers;
mod sitcase;
mod sitcmd;
mod sitcomment;
mod sitextent;
mod sitfor;
mod sitmagic;
mod sitrvalue;
mod sitstrdq;
mod sitstrphantom;
mod sitstrsqesc;
mod sittest;
mod situntilbyte;
mod sitvarbrace;
mod sitvarident;
mod sitvec;
