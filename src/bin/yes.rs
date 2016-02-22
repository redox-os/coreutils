#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{Write, stdout};

use coreutils::extra::{OptionalExt};

fn main() {
    let mut stdout = stdout();

    let answer = env::args().skip(1).next();
    if let Some(x) = answer {
        let print = x.as_bytes();
        loop {
            stdout.write(print).try();
        }
    } else {
        loop {
            stdout.write(b"y").try();
        }
    }; // Dafuq, borrowck
}
