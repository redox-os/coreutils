#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::fs::File;
use std::io::{stdout, stderr, copy};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("ps"), MAN_PAGE);

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut file = File::open("sys:/context").try(&mut stderr);
    copy(&mut file, &mut stdout).try(&mut stderr);
}
