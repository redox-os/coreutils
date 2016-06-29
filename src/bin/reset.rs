#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;

use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{reset} */ r#"
NAME
    reset - terminal initialization

SYNOPSIS
    reset [ -h | --help]

DESCRIPTION
    Initialize the terminal, clearing the screen and setting all parameters to their default values

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for arg in env::args().skip(1){
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        }
    }

    let _ = stdout.write(b"\x1Bc");
}
