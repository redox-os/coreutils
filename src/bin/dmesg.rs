#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, copy, Write};
use std::process::exit;

use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ps} */ r#"
NAME
    dmesg - display the system message buffer

SYNOPSIS
    dmesg [ -h | --help]

DESCRIPTION
    Displays the contents of the system message buffer.

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

    let mut file = File::open("syslog:").try(&mut stderr);
    copy(&mut file, &mut stdout).try(&mut stderr);
}
