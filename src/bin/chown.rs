//#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate syscall;

use std::env;
use std::io::{self, Write};
use std::process::exit;
use arg_parser::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{chown} */ r#"
NAME
    chown - set user and/or group ownership of a file

SYNOPSIS
    chown [-h | --help] [OWNER][:[GROUP]] FILE...

DESCRIPTION
    Set the user and/or group ownership of a file

EXAMPLE
    chown 1000:1000 file

OPTIONS
    -h
    --help
        display this help and exit

AUTHOR
    Written by Jeremy Soller.
"#; /* @MANEND */

const MISSING_OPERAND: &'static str = "missing operand\n";
const HELP_INFO:       &'static str = "Try 'chown --help' for more information.\n";

fn chown(path: &str, uid: u32, gid: u32) -> syscall::Result<usize> {
    let fd = syscall::open(path, syscall::O_STAT)?;
    let res = syscall::fchown(fd, uid, gid);
    let _ = syscall::close(fd);
    res
}

fn main() {
    let stdout     = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.len() >= 2 {
        let mut args = parser.args.iter();

        let arg = args.next().unwrap();

        let mut parts = arg.splitn(2, ":");

        let uid = if let Some(part) = parts.next() {
            if part.is_empty() {
                -1i32 as u32
            } else {
                match part.parse() {
                    Ok(id) => id,
                    Err(err) => {
                        let _ = writeln!(stderr, "chown: failed to parse uid {}: {}", part, err);
                        exit(1);
                    }
                }
            }
        } else {
            -1i32 as u32
        };

        let gid = if let Some(part) = parts.next() {
            if part.is_empty() {
                -1i32 as u32
            } else {
                match part.parse() {
                    Ok(id) => id,
                    Err(err) => {
                        let _ = writeln!(stderr, "chown: failed to parse gid {}: {}", part, err);
                        exit(1);
                    }
                }
            }
        } else {
            -1i32 as u32
        };

        for arg in args {
            if let Err(err) = chown(arg, uid, gid) {
                let _ = writeln!(stderr, "chown: failed to set uid and gid of {} to {} and {}: {}", arg, uid, gid, err);
                exit(1);
            }
        }
    } else {
        stderr.write(MISSING_OPERAND.as_bytes()).try(&mut stderr);
        stderr.write(HELP_INFO.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }
}
