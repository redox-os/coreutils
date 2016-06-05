//#![deny(warnings)]

extern crate extra;

use std::fs;
use std::io;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{shutdown} */ r#"
NAME
    shutdown - stop the system

SYNOPSIS
    shutdown

DESCRIPTION
    Attempt to shutdown the system using ACPI. Failure will be logged to the terminal
"#; /* @MANEND */

fn main() {
    let mut stderr = io::stderr();
    fs::File::create("acpi:off").try(&mut stderr);
}
