#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    let stdout = io::stdout();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stdout.lock());
    }

    for ref path in env::args().skip(1) {
        fs::remove_file(path).try(&mut stdout.lock());
    }
}
