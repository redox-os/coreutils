#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate filetime;
#[macro_use]
extern crate coreutils;

use std::env;
use std::fs::File;
use std::io::stderr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;
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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("touch"), MAN_PAGE);
    parser.process_no_argument();

    let mut stderr = stderr();

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
