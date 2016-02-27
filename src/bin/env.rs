#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, stderr, Write};

use coreutils::extra::{WriteExt, OptionalExt};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for (key, value) in env::vars() {
        stdout.write(key.as_bytes()).try(&mut stderr);
        stdout.write(b"=").try(&mut stderr);
        stdout.writeln(value.as_bytes()).try(&mut stderr);
    }
}
