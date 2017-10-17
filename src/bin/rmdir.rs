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

const MAN_PAGE: &'static str = /* @MANSTART{rmdir} */ r#"
NAME
    rmdir - delete directories

SYNOPSIS
    rmdir [ -h | --help ] DIRECTORY...

DESCRIPTION
    The rmdir utility deletes the directory named by the DIRECTORY operand. Multiple directories can be passed.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("rmdir"), MAN_PAGE);
    parser.process_no_argument();

    let mut stderr = stderr();

    for path in &parser.args {
        fs::remove_dir(path).try(&mut stderr);
    }
}
