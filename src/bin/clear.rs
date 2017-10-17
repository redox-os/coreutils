#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::io::{stdout, Write};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;

const MAN_PAGE: &'static str = /* @MANSTART{clear} */ r#"
NAME
    clear - clear the terminal screen

SYNOPSIS
    clear [ -h | --help]

DESCRIPTION
    Clear the screen, if possible

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("clear"), MAN_PAGE);

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let _ = stdout.write(b"\x1B[2J");
}
