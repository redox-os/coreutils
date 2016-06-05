//#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr};

use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{realpath} */ r#"
NAME
    realpath - return the canonicalized absolute pathname

SYNOPSIS
    realpath FILE...

DESCRIPTION
    realpath gets the aboslute pathname of FILE(s) and prints them out on seperate lines
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    for ref path in env::args().skip(1) {
        let file = fs::canonicalize(path).try(&mut stderr);
        stdout.writeln(file.to_str().try(&mut stderr).as_bytes()).try(&mut stderr);
    }
}
