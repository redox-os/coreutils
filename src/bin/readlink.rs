#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::fs;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;

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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("readlink"), MAN_PAGE);

    for path in &parser.args[0..] {
        println!("{}", fs::read_link(path).unwrap().display());
    }
}
