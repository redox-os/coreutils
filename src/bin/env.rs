#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use extra::io::WriteExt;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{env} */ r#"
NAME
    env - print environment variables and their values

SYNOPSIS
    env [ -h | --help ]

DESCRIPTION
    env prints out the names and values of the variables in the environment, with one name=value pair per line.

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

    for (key, value) in env::vars() {
        stdout.write(key.as_bytes()).try(&mut stderr);
        stdout.write(b"=").try(&mut stderr);
        stdout.writeln(value.as_bytes()).try(&mut stderr);
    }
}
