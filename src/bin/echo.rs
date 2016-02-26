#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::print;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let mut args = env::args();
    let mut newline = true;

    if let Some(arg) = args.nth(1) {
        if arg == "-n" {
            newline = false;
            if let Some(arg) = args.nth(0) {
                print(arg.as_bytes(), &mut stdout);
            }
        } else {
            print(arg.as_bytes(), &mut stdout);
            print(b" ", &mut stdout);
        }
    }
    for arg in args {
        print(b" ", &mut stdout);
        print(arg.as_bytes(), &mut stdout);
    }
    if newline {
        print(b"\n", &mut stdout);
    }
}
