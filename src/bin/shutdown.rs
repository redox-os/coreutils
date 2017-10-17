#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate syscall;
#[macro_use]
extern crate coreutils;

use std::io::{stderr, Error};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"])
        .add_flag(&["r", "reboot"]);
    parser.process_common(help_info!("shutdown"), MAN_PAGE);

    let mut stderr = stderr();

    if parser.found("reboot") {
        syscall::kill(1, SIGTERM).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
    } else {
        syscall::kill(1, SIGKILL).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
    }
}
