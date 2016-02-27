#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io;
use coreutils::extra::OptionalExt;

fn main() {
    let paths = env::args().skip(1);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if paths.len() == 0 {
        io::copy(&mut stdin, &mut stdout).try(&mut stdout);
    } else {
        for path in paths {
            if path == "-" {
                io::copy(&mut stdin, &mut stdout).try(&mut stdout);
            } else {
                let file = fs::File::open(&path);
                let mut file = file.try(&mut stdout);

                io::copy(&mut file, &mut stdout).try(&mut stdout);
            };
        }
    }
}
