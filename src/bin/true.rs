#![deny(warnings)]

extern crate arg_parser;
#[macro_use]
extern crate coreutils;

use std::process;
use std::env;
use arg_parser::ArgParser;
use coreutils::arg_parser::{print_man_page, print_help};

const MAN_PAGE: &'static str = /* @MANSTART{true} */ r#"NAME
    true - do nothing, successfully

SYNOPSIS
    true

DESCRIPTION
    Exit with a status code indicating success.
"#; /* @MANEND */

fn main() {
    if env::args().len() > 1 {
        let mut parser = ArgParser::new(1).add_flag(&["h", "help"]);
        parser.parse(env::args());
        if let Err(err) = parser.found_invalid() {
            print_help(&err, help_info!("true"));
        }
        else if parser.found("help") {
            print_man_page(MAN_PAGE);
        }
    }
    process::exit(0);
}
