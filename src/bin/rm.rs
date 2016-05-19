#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use extra::io::fail;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{rm} */ r#"
NAME
    rm - delete files

SYNOPSIS
    rm [ -h | --help ] FILE...

DESCRIPTION
    The rm utility deletes the file named by the FILE operand. Multiple file names can be passed.

OPTIONS
    --help, -h
        print this message
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
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

    for ref path in env::args().skip(1) {
        fs::remove_file(path).try(&mut stderr);
    }
}
