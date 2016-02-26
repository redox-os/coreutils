#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::stdout;

use coreutils::extra::{OptionalExt, println};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let pwd = env::current_dir().try(&mut stdout);

    let b = pwd.to_str().fail("invalid unicode.", &mut stdout).as_bytes();
    println(b, &mut stdout);
}
