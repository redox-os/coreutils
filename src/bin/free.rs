#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stderr, stdout, copy, Write};
use extra::option::OptionalExt;
use std::process::exit;

const MAN_PAGE: &'static str = /* @MANSTART{free} */ r#"
NAME
    free - display amount of free and used memory in the system

SYNOPSIS
    free [ -h | --help]

DESCRIPTION
    Displays the total amount of free and used physical memory in the system

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

    let mut file = File::open("memory:").try(&mut stderr);
    copy(&mut file, &mut stdout).try(&mut stderr);
}
