#![deny(warnings)]

extern crate arg_parser;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use arg_parser::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{pwd} */ r#"
NAME
    pwd - print working directory

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
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    let pwd = env::current_dir().try(&mut stderr);

    let b = pwd.to_str().fail("invalid unicode.", &mut stderr).as_bytes();
    stdout.write(b).try(&mut stderr);
    stdout.write(&[b'\n']).try(&mut stderr);
}
