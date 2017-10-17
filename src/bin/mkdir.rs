#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::fs;
use std::io::stderr;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{mkdir} */ r#"
NAME
    mkdir - make directories

SYNOPSIS
    mkdir [ -h | --help ] DIRECTORIES...

DESCRIPTION
    The mkdir utility creates the directories named as operands.

OPTIONS
    --help, -h
        print this message
    -p, --parents
        no error if existing, make parent directories as needed
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(2)
        .add_flag(&["h", "help"])
        .add_flag(&["p", "parents"]);
    parser.process_common(help_info!("mkdir"), MAN_PAGE);
    parser.process_no_argument();

    let mut stderr = stderr();

    let mut parents = false;
    if parser.found("parents") {
        parents = true;
    }

    for ref path in &parser.args {
        if parents {
            fs::create_dir_all(path).try(&mut stderr);
        } else {
            fs::create_dir(path).try(&mut stderr);
        }
    }
}
