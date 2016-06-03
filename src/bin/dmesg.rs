#![deny(warnings)]

extern crate extra;

use std::fs::File;
use std::io;

use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ps} */ r#"
NAME
    dmesg - display the system message buffer

SYNOPSIS
    dmesg

DESCRIPTION
    Displays the contents of the system message buffer.
"#; /* @MANEND */

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut file = File::open("syslog:").try(&mut stderr);
    io::copy(&mut file, &mut stdout).try(&mut stderr);
}
