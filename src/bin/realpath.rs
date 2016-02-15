extern crate coreutils;

use std::env;
use std::fs;
use std::process;
use std::io::{Write, stdout};

use coreutils::extra::{OptionalExt, fail};

fn main() {
    if env::args().count() < 2 {
        fail("no arguments.");
    }

    let mut stdout = stdout();

    for ref path in env::args().skip(1) {
        stdout.write(fs::canonicalize(path).try().into_os_string().into_string().unwrap().as_bytes()).try();
        stdout.write(b"\n").try();
    }
}
