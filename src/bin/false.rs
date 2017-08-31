#![deny(warnings)]

extern crate arg_parser;

use std::process;
use std::env;
use std::io::{self, Write};
use arg_parser::ArgParser;

const MAN_PAGE: &'static str = /* @MANSTART{false} */ r#"NAME
    false - do nothing, unsuccessfully

SYNOPSIS
    false

DESCRIPTION
    Exit with a status code indicating failure.
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
    process::exit(1);
}
