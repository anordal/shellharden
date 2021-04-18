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
use crate::machine::OutputSelector;

mod errfmt;
mod filestream;
mod situation;
mod syntaxerror;

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
		osel: OutputSelector::DIFF,
		syntax: true,
		replace: false,
	};

	let mut exit_code: i32 = 0;
	let mut opt_trigger: &str = "-";
	for arg in args {
		if let Some(comparable) = arg.to_str() {
			if comparable.starts_with(opt_trigger) {
				match comparable {
					"--suggest" => {
						sett.osel = OutputSelector::DIFF;
						sett.syntax = false;
						sett.replace = false;
						continue;
					}
					"--syntax" => {
						sett.osel = OutputSelector::ORIGINAL;
						sett.syntax = true;
						sett.replace = false;
						continue;
					}
					"--syntax-suggest" => {
						sett.osel = OutputSelector::DIFF;
						sett.syntax = true;
						sett.replace = false;
						continue;
					}
					"--transform" => {
						sett.osel = OutputSelector::TRANSFORM;
						sett.syntax = false;
						sett.replace = false;
						continue;
					}
					"--check" => {
						sett.osel = OutputSelector::CHECK;
						sett.syntax = false;
						sett.replace = false;
						continue;
					}
					"--replace" => {
						sett.osel = OutputSelector::TRANSFORM;
						sett.syntax = false;
						sett.replace = true;
						continue;
					}
					"--help" | "-h" => {
						help();
						continue;
					}
					"--version" => {
						println!(env!("CARGO_PKG_VERSION"));
						continue;
					}
					"--" => {
						opt_trigger = "\x00";
						continue;
					}
					_ => {
						errfmt::blame_path(&arg, "No such option.");
						exit_code = 3;
						break;
					}
				}
			}
		}
		if let Err(e) = machine::treatfile(&arg, &sett) {
			println!("\x1b[m");
			exit_code = 1;
			match e {
				machine::Error::Stdio(ref fail) => {
					errfmt::blame_path_io(&arg, &fail);
				}
				machine::Error::Syntax(ref fail) => {
					errfmt::blame_syntax(&arg, &fail);
				}
				machine::Error::Check => {
					exit_code = 2;
					break;
				}
			};
		}
	}
	process::exit(exit_code);
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
mod sitrvalue;
mod sitstrdq;
mod sitstrphantom;
mod sitstrsqesc;
mod situntilbyte;
mod sitvarbrace;
mod sitvarident;
mod sitvec;
