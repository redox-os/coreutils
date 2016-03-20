#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io;
use extra::option::OptionalExt;

fn main() {
    let paths = env::args().skip(1);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    if paths.len() == 0 {
        io::copy(&mut stdin, &mut stdout).try(&mut stderr);
    } else {
        for path in paths {
            if path == "-" {
                io::copy(&mut stdin, &mut stdout).try(&mut stderr);
            } else {
                let file = fs::File::open(&path);
                let mut file = file.try(&mut stderr);

                io::copy(&mut file, &mut stdout).try(&mut stderr);
            };
        }
    }
}
