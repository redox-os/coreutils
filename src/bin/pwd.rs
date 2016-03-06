#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, stderr, Write};

use coreutils::extra::OptionalExt;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let pwd = env::current_dir().try(&mut stderr);

    let b = pwd.to_str().fail("invalid unicode.", &mut stderr).as_bytes();
    stdout.write(b).try(&mut stderr);
    stdout.write(&[b'\n']).try(&mut stderr);
}
