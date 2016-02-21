#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::time::Duration;
use std::thread;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    if let Some(arg) = env::args().nth(1) {
        thread::sleep(Duration::new(arg.parse().try(), 0))
    } else {
        fail("missing argument.");
    }
}
