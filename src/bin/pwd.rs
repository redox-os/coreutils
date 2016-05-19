#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};

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

    if env::args().count() == 2 {
        if let Some(arg) = env::args().nth(1) {
            if arg == "--help" || arg == "-h" {
                stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                return;
            }
        }
    }

    let pwd = env::current_dir().try(&mut stderr);

    let b = pwd.to_str().fail("invalid unicode.", &mut stderr).as_bytes();
    stdout.write(b).try(&mut stderr);
    stdout.write(&[b'\n']).try(&mut stderr);
}
