#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};

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

    if env::args().count() == 2 {
        if let Some(arg) = env::args().nth(1) {
            if arg == "--help" || arg == "-h" {
                stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                return;
            }
        }
    }

    if env::args().count() < 2 {
        fail("missing value.", &mut stderr);
    }

    let max: u32 = match std::env::args().nth(1).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => fail("invalid value: please provide a valid, unsigned integer.", &mut stderr),
    };

    for i in 1..max + 1 {
        stdout.writeln(i.to_string().as_bytes()).try(&mut stderr);
    }
}
