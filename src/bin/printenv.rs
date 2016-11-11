#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use coreutils::ArgParser;
use extra::io::WriteExt;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{printenv} */ r#"
NAME
    printenv - print environment variables

SYNOPSIS
    printenv [-h | --help] VARIABLES...

DESCRIPTION
    Print the values of the specified environment VARIABLES.

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

    if parser.args.is_empty() {
        stderr.write(b"Please provide a variable name\n").try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }
    if parser.flagged(&'h') || parser.flagged("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    for arg in &parser.args {
        let value = env::var(arg).try(&mut stderr);
        stdout.writeln(value.as_bytes()).try(&mut stderr);
    }
    exit(0);
}
