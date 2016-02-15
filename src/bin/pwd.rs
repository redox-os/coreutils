extern crate coreutils;

use std::env;
use std::io::{Write, stdout};

use coreutils::extra::OptionalExt;

fn main() {
    let mut stdout = stdout();
    stdout.write(env::current_dir().try().into_os_string().into_string().unwrap().as_bytes());
    stdout.write(b"\n");
}
