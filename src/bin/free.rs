#![deny(warnings)]

extern crate coreutils;
use std::fs::File;
use std::io;

use coreutils::extra::OptionalExt;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut file = File::open("memory:").try(&mut stdout);
    let res = io::copy(&mut file, &mut stdout);
    res.try(&mut stdout);
}
