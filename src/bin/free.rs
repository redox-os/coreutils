#![deny(warnings)]

extern crate coreutils;
use std::fs::File;
use std::io;

use coreutils::extra::OptionalExt;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut file = File::open("memory:").try(&mut stderr);
    io::copy(&mut file, &mut stdout).try(&mut stderr);
}
