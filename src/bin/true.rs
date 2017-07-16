#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;

use std::process;
use std::env;
use std::io::{self, Write};
use arg_parser::ArgParser;

const MAN_PAGE: &'static str = /* @MANSTART{true} */ r#"NAME
    true - do nothing, successfully

SYNOPSIS
    true

DESCRIPTION
    Exit with a status code indicating success.
"#; /* @MANEND */

fn main() {
    if env::args().len() > 1 {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        let mut parser = ArgParser::new(1).add_flag(&["h", "help"]);
        parser.parse(env::args());
        if parser.found("help") {
            stdout.write(MAN_PAGE.as_bytes()).ok();
            stdout.flush().ok();
        }
    }
    process::exit(0);
}
