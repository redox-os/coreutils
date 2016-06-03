#![deny(warnings)]

extern crate extra;

use std::fs::File;
use std::io;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{free} */ r#"
NAME
    free - display amount of free and used memory in the system

SYNOPSIS
    free

DESCRIPTION
    Displays the total amount of free and used physical memory in the system
"#; /* @MANEND */

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut file = File::open("memory:").try(&mut stderr);
    io::copy(&mut file, &mut stdout).try(&mut stderr);
}
