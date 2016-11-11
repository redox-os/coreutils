#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stderr, stdout, Write};
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{shutdown} */ r#"
NAME
    shutdown - stop the system

SYNOPSIS
    shutdown [ -h | -help ]

DESCRIPTION
    Attempt to shutdown the system using ACPI. Failure will be logged to the terminal

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

    if parser.flagged(&'h') || parser.flagged("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    fs::File::create("acpi:off").try(&mut stderr);
}
