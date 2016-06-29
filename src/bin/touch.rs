#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = /* @MANSTART{touch} */ r#"
NAME
    touch - create file(s)

SYNOPSIS
    touch [ -h | --help ] FILE...

DESCRIPTION
    Create the FILE(s) arguments provided

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

    // TODO update file modification date/time

    for arg in env::args().skip(1) {
        File::create(&arg).try(&mut stderr);
    }
}
