#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use coreutils::ArgParser;
use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{realpath} */ r#"
NAME
    realpath - return the canonicalized absolute pathname

SYNOPSIS
    realpath [ -h | --help ] FILE...

DESCRIPTION
    realpath gets the absolute pathname of FILE(s) and prints them out on seperate lines

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged(&'h') || parser.flagged("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.is_empty() {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

    for path in &parser.args {
        let file = fs::canonicalize(path).try(&mut stderr);
        stdout.writeln(file.to_str().try(&mut stderr).as_bytes()).try(&mut stderr);
    }
}
