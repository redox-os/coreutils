#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate syscall;

use std::env;
use std::io::{stderr, stdout, Error, Write};
use std::process::exit;
use arg_parser::ArgParser;
use extra::option::OptionalExt;
use syscall::flag::{SIGTERM, SIGKILL};

const MAN_PAGE: &'static str = /* @MANSTART{shutdown} */ r#"
NAME
    shutdown - stop the system

SYNOPSIS
    shutdown [ -h | --help ] [ -r | --reboot ]

DESCRIPTION
    Attempt to shutdown the system using ACPI. Failure will be logged to the terminal

OPTIONS
    -h
    --help
        display this help and exit

    -r
    --reboot
        reboot instead of powering off
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"])
        .add_flag(&["r", "reboot"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.found("reboot") {
        syscall::kill(1, SIGTERM).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
    } else {
        syscall::kill(1, SIGKILL).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
    }
}
