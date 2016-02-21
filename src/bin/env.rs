#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{stdout, Write};

use coreutils::extra::OptionalExt;

fn main() {
    let mut stdout = stdout();

    for (key, value) in env::vars() {
        writeln!(stdout, "{}={}", key, value).try();
    }
}
