#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    let mut stderr = io::stderr();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    for ref path in env::args().skip(1) {
        fs::create_dir(path).try(&mut stderr);
    }
}
