extern crate anyhow;
extern crate arg_parser;
extern crate libredox;

use std::env;
use std::io::{stdout, Write};
use arg_parser::ArgParser;
use libredox::flag::{SIGTERM, SIGKILL};

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

fn main() -> anyhow::Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"])
        .add_flag(&["r", "reboot"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes())?;
        stdout.flush()?;
        return Ok(());
    }

    if parser.found("reboot") {
        libredox::call::kill(1, SIGTERM as _)?;
    } else {
        libredox::call::kill(1, SIGKILL as _)?;
    }
    Ok(())
}
