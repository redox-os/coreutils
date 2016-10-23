#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{self, stderr, stdout, Write};
use std::path;
use std::process::exit;
use coreutils::{ArgParser, Flag};
use extra::io::fail;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{cp} */ r#"
NAME
    cp - copy files

SYNOPSIS
    cp SOURCE_FILE(S) DESTINATION_FILE...

DESCRIPTION
    The cp utility copies the contents of the SOURCE_FILE to the DESTINATION_FILE. If multiple
    source files are specified, then they are copied to DESTINATION_FILE.

OPTIONS
    -h
    --help
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

    if parser.args.is_empty() {
        fail("No source argument. Use --help to see the usage.", &mut stderr);
    }
    else if parser.args.len() == 1 {
        fail("No destination arguments. Use --help to see the usage.", &mut stderr);
    }
    else if parser.args.len() == 2 {
        let mut src_file = fs::File::create(&parser.args[0]).try(&mut stderr);
        let mut dst_file = fs::File::create(&parser.args[1]).try(&mut stderr);
        io::copy(&mut src_file, &mut dst_file).try(&mut stderr);
    }
    else {
        // This unwrap won't panic since it's been verified not to be empty
        let dst = parser.args.pop().unwrap();
        let dst = path::Path::new(&dst);
        if dst.is_dir() {
            for ref arg in parser.args {
                let src = path::Path::new(arg);
                fs::copy(src, dst).try(&mut stderr);
            }
        }
        else if dst.is_file() {
            fail("Destination should be a path, not a file. Use --help to see the usage.", &mut stderr);
        }
        else {
            fail("No destination found. Use --help to see the usage.", &mut stderr);
        }
    }
}
