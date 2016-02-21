#![deny(warnings)]

extern crate coreutils;

use std::fs;

use coreutils::extra::OptionalExt;

fn main() {
    println!("Good bye!");
    fs::File::create("acpi:off").try();
}
