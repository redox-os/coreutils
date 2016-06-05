//#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs::File;
use std::io;
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = /* @MANSTART{touch} */ r#"
NAME
    touch - create file(s)

SYNOPSIS
    touch FILE...

DESCRIPTION
    Create the FILE(s) arguments provided
"#; /* @MANEND */

fn main() {
    let mut stderr = io::stderr();

    if env::args().count() < 2 {
        fail("no arguments.", &mut stderr);
    }

    // TODO update file modification date/time

    for arg in env::args().skip(1) {
        File::create(&arg).try(&mut stderr);
    }
}
