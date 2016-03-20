#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr};

use extra::io::WriteExt;
use extra::option::OptionalExt;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let answer = env::args().skip(1).next();
    if let Some(x) = answer {
        let print = x.as_bytes();
        loop {
            stdout.writeln(print).try(&mut stderr);
        }
    } else {
        loop {
            stdout.writeln(b"y").try(&mut stderr);
        }
    }; // Dafuq, borrowck
}
