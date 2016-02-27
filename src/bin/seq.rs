#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, stderr};

use coreutils::extra::{fail, WriteExt, OptionalExt};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stderr = stderr();
    let mut stderr = stderr.lock();

    if env::args().count() < 2 {
        fail("missing value.", &mut stderr);
    }

    let max: u32 = match std::env::args().nth(1).map(|a| a.parse()) {
        Some(Ok(n)) if n > 0 => n,
        _ => fail("invalid value: please provide a valid, unsigned integer.", &mut stderr),
    };

    for i in 1..max + 1 {
        stdout.writeln(i.to_string().as_bytes()).try(&mut stderr);
    }
}
