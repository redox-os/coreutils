#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::io::WriteExt;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{yes} */ r#"
NAME
    yes - print the letter y or a given word, forever.

SYNOPSIS
    yes [ [ -h | --help ] | [ word ] ]

DESCRIPTION
    The yes utility prints the word passed as an operand, forever. If no operand is passed, then
    it defaults to the letter 'y'.

OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1, 0)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.enabled_flag('h') || parser.enabled_flag("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    if parser.args.is_empty() {
        loop {
            stdout.writeln(b"y").try(&mut stderr);
        }
    }
    else {
        let answer = parser.args.join(" ");
        let print = answer.as_bytes();
        loop {
            stdout.writeln(print).try(&mut stderr);
        }
    }
}
