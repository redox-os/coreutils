#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{pwd} */ r#"
NAME
    pwd - return working directory name

SYNOPSIS
    pwd [ -h | --help ]

DESCRIPTION
    The pwd utility writes the absolute pathname of the current working directory to the standard output.

OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged('h') || parser.flagged("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    let pwd = env::current_dir().try(&mut stderr);

    let b = pwd.to_str().fail("invalid unicode.", &mut stderr).as_bytes();
    stdout.write(b).try(&mut stderr);
    stdout.write(&[b'\n']).try(&mut stderr);
}
