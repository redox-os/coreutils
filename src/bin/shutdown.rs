#![deny(warnings)]

extern crate extra;

use std::fs;
use std::io;

use extra::io::WriteExt;
use extra::option::OptionalExt;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    stdout.writeln(b"Good bye!").try(&mut stderr);
    fs::File::create("acpi:off").try(&mut stderr);
}
