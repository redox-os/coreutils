#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::io::fail;
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
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged('h') || parser.flagged("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    if parser.args.is_empty() {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }

    for path in &parser.args {
        fs::remove_dir(path).try(&mut stderr);
    }
}
