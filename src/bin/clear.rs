extern crate arg_parser;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use arg_parser::ArgParser;
use extra::option::OptionalExt;

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
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let _ = stdout.write(b"\x1B[2J");
}
