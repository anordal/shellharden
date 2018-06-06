use std::process::Command;

// Find the executable (https://github.com/rust-lang/cargo/issues/3670)
const BASH_RUN_MODULETEST : &str = "
	shopt -s failglob globstar\n\
	if ! [[ -v CARGO_TARGET_DIR ]]; then\n\
		CARGO_TARGET_DIR=.\n\
	fi\n\
	for exe in \"$CARGO_TARGET_DIR\"/**/shellharden; do\n\
		if test -f \"$exe\" && test -x \"$exe\"; then\n\
			moduletests/run \"$exe\" moduletests/original/*\n\
		fi\n\
	done\n\
";

#[test]
fn moduletest() {
	let mut child = Command::new("bash")
		.arg("-ec")
		.arg(BASH_RUN_MODULETEST)
		.spawn()
		.expect("moduletests/run: Command not found")
	;
	match &child.wait() {
		&Ok(status) => assert!(status.success()),
		&Err(_) => assert!(false),
	}
}
