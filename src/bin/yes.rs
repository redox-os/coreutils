#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::io::{stdout, stderr};
use arg_parser::ArgParser;
use extra::io::WriteExt;
use extra::option::OptionalExt;
use coreutils::arg_parser::ArgParserExt;

const MAN_PAGE: &'static str = /* @MANSTART{yes} */ r#"
NAME
    yes - print the letter y or a given word, forever.

SYNOPSIS
    yes [ [ -h | --help ] | [ word ] ]

DESCRIPTION
    The yes utility prints the word passed as an operand, forever. If no operand is passed, then
    it defaults to the letter 'y'.

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
    parser.process_common(help_info!("yes"), MAN_PAGE);

    let answer = if parser.args.is_empty() {
        "y".to_owned()
    } else {
        parser.args.join(" ")
    };
    let print = answer.as_bytes();
    loop {
        stdout.writeln(print).try(&mut stderr);
    }
}
