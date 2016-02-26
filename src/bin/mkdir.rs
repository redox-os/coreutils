#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stdout);
    }

    for ref path in env::args().skip(1) {
        fs::create_dir(path).try(&mut stdout);
    }
}
