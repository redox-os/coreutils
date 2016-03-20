#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{self, stderr};
use extra::option::OptionalExt;

fn main() {
    let mut stderr = stderr();
    let ref src = env::args().nth(1).fail("no source argument.", &mut stderr);
    let ref dst = env::args().nth(2).fail("no destination argument.", &mut stderr);

    let mut src_file = fs::File::open(src).try(&mut stderr);
    let mut dst_file = fs::File::create(dst).try(&mut stderr);

    io::copy(&mut src_file, &mut dst_file).try(&mut stderr);
    drop(src_file); // close file

    fs::remove_file(src).try(&mut stderr);
}
