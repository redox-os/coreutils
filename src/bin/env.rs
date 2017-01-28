#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{env} */ r#"
NAME
    env - print environment variables and their values

SYNOPSIS
    env [ -h | --help ]

DESCRIPTION
    env prints out the names and values of the variables in the environment, with one name=value pair per line.

OPTIONS
    -h
    --help
        Print this manual page.
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

    let mut string = String::new();
    for (key, value) in env::vars() {
        string.push_str(&key);
        string.push('=');
        string.push_str(&value);
        string.push('\n')
    }
    stdout.write(string.as_bytes()).try(&mut stderr);
}
