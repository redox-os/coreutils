#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use coreutils::ArgParser;
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
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged('h') || parser.flagged("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let _ = stdout.write(b"\x1Bc");
}
