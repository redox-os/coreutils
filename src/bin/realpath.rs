#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::fs;
use std::io::{stdout, stderr};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::io::{WriteExt};
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
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("realpath"), MAN_PAGE);
    parser.process_no_argument();

    for path in &parser.args {
        let file = fs::canonicalize(path).try(&mut stderr);
        stdout.writeln(file.to_str().try(&mut stderr).as_bytes()).try(&mut stderr);
    }
}
