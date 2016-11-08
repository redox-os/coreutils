#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::{fail, WriteExt};

const MAN_PAGE: &'static str = /* @MANSTART{seq} */ r#"
NAME
    seq - print sequence of numbers

SYNOPSIS
    seq [ -h | --help ] last

DESCRIPTION
    The seq utility prints a sequence of numbers, in increments of 1, one number per line,
    from 1 to the number provided as the last operand.

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

    if parser.args.is_empty() {
        fail("missing value.", &mut stderr);
    }

    let max: u32 = match parser.args.get(0).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => fail("invalid value: please provide a valid, unsigned integer.", &mut stderr),
    };

    for i in 1..max + 1 {
        stdout.writeln(i.to_string().as_bytes()).try(&mut stderr);
    }
}
