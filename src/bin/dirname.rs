#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{dirname} */ r#"
NAME
    dirname - strip last component from file name

SYNOPSIS
    dirname [ -h | --help ] FILE...

DESCRIPTION
    Strip last component from file name.

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

    for path in &parser.args[0..] {
        if path.len() != 0 && path.chars().all(|c| c == '/') {
            println!("/");
        } else {
            let path = path.trim_right_matches('/');
            if let Some(end) = path.rfind('/') {
                println!("{}", &path[..end]);
            } else {
                println!(".");
            }
        }
    }
}
