extern crate arg_parser;
extern crate extra;

use arg_parser::ArgParser;
use extra::option::OptionalExt;
use std::env;
use std::io::{stderr, stdout, Write};
use std::process::{exit, ExitCode};

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

fn main() -> ExitCode {
    let mut not_found_count = 0;
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
        stderr
            .write(b"Please provide a program name\n")
            .try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }

    let paths = env::var("PATH").unwrap();

    for program in parser.args.iter() {
        let mut executable_path = None;

        for mut path in env::split_paths(&paths) {
            path.push(program);
            if path.exists() {
                executable_path = Some(path);
                break;
            }
        }

        if let Some(path) = executable_path {
            let _ = writeln!(stdout, "{}", path.display());
        } else {
            let _ = writeln!(stderr, "{} not found", program);
            not_found_count += 1;
        }
    }
    if not_found_count == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(not_found_count)
    }
}
