#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stderr, stdout, Write};
use std::process::exit;
use coreutils::{ArgParser, Flag};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{mv} */ r#"
NAME
    mv - move files

SYNOPSIS
    mv [ -h | --help ] SOURCE_FILE DESTINATION_FILE

DESCRIPTION
    The mv utility renames the file named by the SOURCE_FILE operand to the destination path named by the DESTINATION_FILE operand.

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

    if parser.enabled_flag(Flag::Long("help")) {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let ref src = parser.args.get(0).fail("No source argument. Use --help to see the usage.", &mut stderr);
    let ref dst = parser.args.get(1).fail("No destination argument. Use --help to see the usage.", &mut stderr);

    fs::rename(src, dst).try(&mut stderr);
}
