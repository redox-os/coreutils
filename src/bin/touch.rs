#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;
extern crate syscall;
extern crate filetime;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::process::exit;
use arg_parser::ArgParser;
use extra::option::OptionalExt;
use extra::io::fail;
use filetime::{set_file_times, FileTime};

const MAN_PAGE: &'static str = /* @MANSTART{touch} */ r#"
NAME
    touch - create file(s)

SYNOPSIS
    touch [ -h | --help ] FILE...

DESCRIPTION
    Create the FILE(s) arguments provided

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
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.is_empty() {
        fail("no arguments.", &mut stderr);
    }
    else {
        // TODO update file modification date/time
        for arg in env::args().skip(1) {
            if Path::new(&arg).is_file() {
                let mtime = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let time = FileTime::from_seconds_since_1970(mtime.as_secs(), mtime.subsec_nanos());
                set_file_times(&arg, time, time).try(&mut stderr);
            } else {
                File::create(&arg).try(&mut stderr);
            }
        }
    }
}
