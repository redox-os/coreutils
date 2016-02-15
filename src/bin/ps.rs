extern crate coreutils;

use std::fs::File;
use std::io;

use coreutils::extra::OptionalExt;

fn main() {
    let mut file = File::open("context:").try();
    io::copy(&mut file, &mut io::stdout()).try();
}
