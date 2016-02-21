#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs::File;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    if env::args().count() < 2 {
        fail("no arguments.");
    }

    // TODO update file modification date/time

    for arg in env::args().skip(1) {
        File::create(&arg).try();
    }
}
