#![deny(warnings)]

extern crate coreutils;

use std::fs;
use std::io;

use coreutils::extra::{OptionalExt, println};

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    println(b"Good bye!", &mut stdout);
    fs::File::create("acpi:off").try(&mut stdout);
}
