#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::{print, println};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    for (key, value) in env::vars() {
        print(key.as_bytes(), &mut stdout);
        print(b"=", &mut stdout);
        println(value.as_bytes(), &mut stdout);
    }
}
