#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::{fail, println};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    if env::args().count() < 2 {
        fail("missing value.", &mut stdout);
    }

    let max: u32 = match std::env::args().nth(1).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => fail("invalid value: please provide a valid, unsigned integer.", &mut stdout),
    };

    for i in 1..max + 1 {
        println(i.to_string().as_bytes(), &mut stdout);
    }
}
