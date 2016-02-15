extern crate coreutils;

use std::env;
use std::fs;
use std::process;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    if env::args().count() < 2 {
        fail("no arguments.");
    }

    for ref path in env::args().skip(1) {
        fs::remove_dir(path).try();
    }
}
