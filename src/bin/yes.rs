extern crate coreutils;

use std::env;
use std::io::{Write, stdout};
use std::thread;

use coreutils::extra::{OptionalExt};

fn main() {
    let mut stdout = stdout();

    let answer = env::args().skip(1).next();
    if let Some(mut x) = answer {
        x.push('\n');
        let print = x.as_bytes();
        loop {
            stdout.write(print).try();
            thread::yield_now();
        }
    } else {
        loop {
            stdout.write(b"y\n").try();
            thread::yield_now();
        }
    }; // Dafuq, borrowck
}
