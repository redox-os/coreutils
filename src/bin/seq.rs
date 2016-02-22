#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{Write, stdout};

use coreutils::extra::{fail, OptionalExt};

fn main() {
    if env::args().count() < 2 {
        fail("missing value.");
    }

    let max: u32 = match std::env::args().nth(1).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => fail("invalid value: please provide a valid, unsigned number."),
    };

    let mut stdout = stdout();

    for i in 1..max + 1 {
        stdout.write(i.to_string().as_bytes()).try();
    }
}
