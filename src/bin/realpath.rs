#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use std::process::exit;

use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{realpath} */ r#"
NAME
    realpath - return the canonicalized absolute pathname

SYNOPSIS
    realpath [ -h | --help ] FILE...

DESCRIPTION
    realpath gets the absolute pathname of FILE(s) and prints them out on seperate lines

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for arg in env::args().skip(1){
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        }
    }

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    for ref path in env::args().skip(1) {
        let file = fs::canonicalize(path).try(&mut stderr);
        stdout.writeln(file.to_str().try(&mut stderr).as_bytes()).try(&mut stderr);
    }
}
