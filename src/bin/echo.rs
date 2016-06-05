//#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{echo} */ r#"
NAME
    echo - display a line of text

SYNOPSIS
    echo [STRING]...

DESCRIPTION
    Print the STRING(s) to standard output.

OPTIONS
    -n
        do not output the trailing newline
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut args = env::args();
    let mut newline = true;

    if let Some(arg) = args.nth(1) {
        if arg == "-n" {
            newline = false;
            if let Some(arg) = args.nth(0) {
                stdout.write(arg.as_bytes()).try(&mut stderr);
            }
        } else {
            stdout.write(arg.as_bytes()).try(&mut stderr);
        }
    }
    for arg in args {
        stdout.write(b" ").try(&mut stderr);
        stdout.write(arg.as_bytes()).try(&mut stderr);
    }
    if newline {
        stdout.write(b"\n").try(&mut stderr);
    }
}
