#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, Write};

use coreutils::extra::OptionalExt;

fn main() {
    let mut stdout = stdout();

    let mut newline = true;
    for arg in env::args().skip(1) {
        if arg == "-n" {
            newline = false;
        } else {
            write!(stdout, "{} ", arg).try();
        }
    }
    if newline {
        writeln!(stdout, "\n").try();
    }
}
