#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs::File;
use std::io;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    let mut stderr = io::stderr();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    // TODO update file modification date/time

    for arg in env::args().skip(1) {
        File::create(&arg).try(&mut stderr);
    }
}
