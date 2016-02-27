#![deny(warnings)]

extern crate coreutils;

use std::fs;
use std::io;

use coreutils::extra::{OptionalExt, WriteExt};

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    stdout.writeln(b"Good bye!").try(&mut stderr);
    fs::File::create("acpi:off").try(&mut stderr);
}
