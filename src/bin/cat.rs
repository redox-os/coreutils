extern crate coreutils;

use std::env;
use std::fs;
use std::io;
use coreutils::extra::OptionalExt;

fn main() {
    let paths = env::args().skip(1);

    if paths.len() == 0 {
        io::copy(&mut io::stdin(), &mut io::stdout()).try();
    } else {
        for path in paths {
            if path == "-" {
                io::copy(&mut io::stdin(), &mut io::stdout()).try();
            } else {
                let mut file = fs::File::open(&path).try();
                io::copy(&mut file, &mut io::stdout()).try();
            };
        }
    }
}
