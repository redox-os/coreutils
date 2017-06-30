#![deny(warnings)]

extern crate coreutils;
extern crate extra;
extern crate syscall;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{readlink} */ r#"
NAME
    readlink - read the contents of a symbolic link

SYNOPSIS
    readlink [ -h | --help ] FILE...

DESCRIPTION
    Read the contents of a symbolic link.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    for path in &parser.args[0..] {
        println!("{}", fs::read_link(path).unwrap().display());
    }
}
