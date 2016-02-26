#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::stdout;

use coreutils::extra::{OptionalExt, fail, println};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stdout);
    }

    for ref path in env::args().skip(1) {
        let file = fs::canonicalize(path).try(&mut stdout);

        let b = file.to_str().try(&mut stdout).as_bytes();
        println(b, &mut stdout);
    }
}
