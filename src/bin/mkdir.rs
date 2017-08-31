#![deny(warnings)]

extern crate arg_parser;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use arg_parser::ArgParser;
use extra::io::fail;
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
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(2)
        .add_flag(&["h", "help"])
        .add_flag(&["p", "parents"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }
    if parser.args.is_empty() {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

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
