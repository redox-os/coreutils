#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::stderr;
use extra::option::OptionalExt;

fn main() {
    let mut stderr = stderr();
    let ref src = env::args().nth(1).fail("no source argument.", &mut stderr);
    let ref dst = env::args().nth(2).fail("no destination argument.", &mut stderr);

    fs::rename(src, dst).try(&mut stderr);
}
