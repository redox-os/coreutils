#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use extra::io::fail;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{mkdir} */ r#"
NAME
    mkdir - make directories

SYNOPSIS
    mkdir DIRECTORIES...

DESCRIPTION
    The mkdir utility creates the directories named as operands.

OPTIONS
    --help
        print this message
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    if env::args().count() < 2 {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

    if env::args().count() == 2 {
        if let Some(arg) = env::args().nth(1) {
            if arg == "--help" {
                stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                return;
            }
        }
    }

    for ref path in env::args().skip(1) {
        fs::create_dir(path).try(&mut stderr);
    }
}
