#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::print;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let mut newline = true;
    for arg in env::args().skip(1) {
        if arg == "-n" {
            newline = false;
        } else {
            print(arg.as_bytes(), &mut stdout);
            print(b" ", &mut stdout);
        }
    }
    if newline {
        print(b"\n", &mut stdout);
    }
}
