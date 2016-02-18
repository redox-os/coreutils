extern crate coreutils;

use std::env;
use std::io::{Write, stdout};

use coreutils::extra::OptionalExt;

fn main() {
    let mut stdout = stdout();
    stdout.write(env::current_dir().try().to_str().unwrap().as_bytes());
    stdout.write(b"\n");
}
