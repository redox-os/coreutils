#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = /* @MANSTART{touch} */ r#"
NAME
    touch - create file(s)

SYNOPSIS
    touch [ -h | --help ] FILE...

DESCRIPTION
    Create the FILE(s) arguments provided

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1, 0)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.enabled_flag('h') || parser.enabled_flag("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.is_empty() {
        fail("no arguments.", &mut stderr);
    }
    else {
        // TODO update file modification date/time
        for arg in env::args().skip(1) {
            File::create(&arg).try(&mut stderr);
        }
    }
}
