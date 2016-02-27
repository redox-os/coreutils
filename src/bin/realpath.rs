#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{stdout, stderr};

use coreutils::extra::{OptionalExt, WriteExt, fail};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    for ref path in env::args().skip(1) {
        let file = fs::canonicalize(path).try(&mut stderr);
        stdout.writeln(file.to_str().try(&mut stderr).as_bytes()).try(&mut stderr);
    }
}
