#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::println;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let answer = env::args().skip(1).next();
    if let Some(x) = answer {
        let print = x.as_bytes();
        loop {
            println(print, &mut stdout);
        }
    } else {
        loop {
            println(b"y", &mut stdout)
        }
    }; // Dafuq, borrowck
}
