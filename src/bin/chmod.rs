#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use std::os::unix::fs::PermissionsExt;
use std::u32;
use coreutils::ArgParser;
use extra::io::fail;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{chmod} */ r#"
NAME
    chmod - change permissions

SYNOPSIS
    chmod [ -h | --help ] MODE FILE...

DESCRIPTION
    The chmod utility changes the permissions of files. Multiple files can be passed.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    if let Some(mode_arg) = parser.args.get(0) {
        let mode = u32::from_str_radix(mode_arg, 8).try(&mut stderr);

        if parser.args.is_empty() {
            fail("No files. Use --help to see the usage.", &mut stderr);
        }

        for path in &parser.args[1..] {
            fs::set_permissions(path, fs::Permissions::from_mode(mode)).try(&mut stderr);
        }
    } else {
        fail("No mode. Use --help to see the usage.", &mut stderr);
    }
}
