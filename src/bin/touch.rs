#![deny(warnings)]

extern crate coreutils;
extern crate extra;
extern crate syscall;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, Error, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::fail;
use syscall::data::TimeSpec;
use syscall::flag::O_WRONLY;

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

                let file = syscall::open(&arg, O_WRONLY).map_err(|e| Error::from_raw_os_error(e.errno)).try(&mut stderr);
                let res = syscall::futimens(file, &[TimeSpec {
                    tv_sec: mtime.as_secs() as i64,
                    tv_nsec: mtime.subsec_nanos() as i32,
                }]).map_err(|e| Error::from_raw_os_error(e.errno));
                let _ = syscall::close(file);
                res.try(&mut stderr);
            } else {
                File::create(&arg).try(&mut stderr);
            }
        }
    }
}
