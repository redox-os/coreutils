#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, Write};

use coreutils::extra::OptionalExt;

fn main() {
    let mut stdout = stdout();

    let mut args = env::args();
    let mut newline = true;

    if let Some(arg) = args.nth(1) {
        if arg == "-n" {
            newline = false;
            if let Some(arg) = args.nth(0) {write!(stdout, "{}", arg).try();}
        } else {
            write!(stdout, "{}", arg).try();
        }
    }
    for arg in args {
        write!(stdout, " {}", arg).try();
    }
    if newline {
        write!(stdout, "\n").try();
    }
}
