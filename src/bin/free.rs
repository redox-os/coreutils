#![deny(warnings)]

extern crate extra;

use std::fs::File;
use std::io;
use extra::option::OptionalExt;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut file = File::open("memory:").try(&mut stderr);
    io::copy(&mut file, &mut stdout).try(&mut stderr);
}
