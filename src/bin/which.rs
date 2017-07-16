#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;

use std::env;
use std::path::Path;
use std::process::exit;
use std::io::{stdout, stderr, Write};
use arg_parser::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = r#"
NAME
    which - locate a command
SYNOPSIS
    which [ -h | --help ]
DESCRIPTION
    which prints the full path of shell commands
OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1).add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.is_empty() {
        stderr.write(b"Please provide a program name\n").try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }

    let mut string = String::new();
    let path = env::var("PATH").unwrap();

    let read_dir = Path::new(&path).read_dir().try(&mut stderr);
    let binaries: Vec<String> = read_dir
        .filter_map(|x| x.ok())
        .map(|dir| {
            let mut file_name = dir.file_name().to_string_lossy().into_owned();
            if dir.file_type().try(&mut stderr).is_dir() {
                file_name.push('/');
            }
            file_name
        })
        .collect();

    for program in parser.args.iter() {
        if binaries.contains(program) {
            string.push_str(&path);
            string.push('/');
            string.push_str(program);
        } else {
            string.push_str(program);
            string.push_str(" not found");
        }
        string.push('\n');
    }

    stdout.write(string.as_bytes()).try(&mut stderr);
}
