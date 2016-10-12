#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use coreutils::{ArgParser, Flag};
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
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.args.is_empty() {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

    if parser.enabled_flag(Flag::Long("help")) {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    for ref path in &parser.args {
        fs::create_dir(path).try(&mut stderr);
    }
}
