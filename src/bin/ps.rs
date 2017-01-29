#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, copy, Write};
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ps} */ r#"
NAME
    ps - report a snapshot of the current processes

SYNOPSIS
    ps [ -h | --help]

DESCRIPTION
    Displays information about processes and threads that are currently active

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
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let mut file = File::open("sys:/context").try(&mut stderr);
    copy(&mut file, &mut stdout).try(&mut stderr);
}
