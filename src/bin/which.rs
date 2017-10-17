#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::env;
use std::process::exit;
use std::io::{stderr, Write};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = r#"
NAME
    which - locate a command
SYNOPSIS
    which [ -h | --help ]
DESCRIPTION
    which prints the full path of shell commands
OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

fn main() {
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1).add_flag(&["h", "help"]);
    parser.process_common(help_info!("which"), MAN_PAGE);

    if parser.args.is_empty() {
        stderr.write(b"Please provide a program name\n").try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }

    let paths = env::var("PATH").unwrap();

    for program in parser.args.iter() {
        let mut executable_path = None;

        for mut path in env::split_paths(&paths) {
            path.push(program);
            if path.exists() {
                executable_path = Some(path);
                break;
            }
        }

        if let Some(path) = executable_path {
            println!("{}", path.display());
        } else {
            println!("{} not found", program);
        }
    }
}
