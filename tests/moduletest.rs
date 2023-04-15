use std::env;
use std::process;
use std::process::Command;

#[test]
fn moduletest() {
	let mut child = Command::new("moduletests/run")
		.arg(env!("CARGO_BIN_EXE_shellharden"))
		.arg("moduletests")
		.spawn()
		.expect("moduletests/run: Command not found")
	;
	match &child.wait() {
		&Ok(waitresult) => {
			if let Some(status) = waitresult.code() {
				process::exit(status);
			}
			assert!(false);
		}
		&Err(_) => assert!(false),
	}
}
