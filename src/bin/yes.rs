#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
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

    if env::args().count() == 2 {
        if let Some(arg) = env::args().nth(1) {
            if arg == "--help" || arg == "-h" {
                stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                return;
            }
        }
    }

    if env::args().count() >= 2 {
        let answer = env::args().skip(1).collect::<Vec<_>>().join(" ");
        let print = answer.as_bytes();
        loop {
            stdout.writeln(print).try(&mut stderr);
        }
    } else {
        loop {
            stdout.writeln(b"y").try(&mut stderr);
        }
    };
}
