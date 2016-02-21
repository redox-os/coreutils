#![deny(warnings)]

extern crate coreutils;
use std::fs::File;
use std::io;

use coreutils::extra::OptionalExt;

fn main() {
    let mut file = File::open("memory:").try();
    io::copy(&mut file, &mut io::stdout()).try();
}
