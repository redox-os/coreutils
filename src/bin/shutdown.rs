extern crate coreutils;

use std::fs;
use std::process::exit;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    println!("Good bye!");
    fs::File::create("acpi:off").try();
}
