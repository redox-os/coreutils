#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write, Stderr};
use std::path::Path;
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{rm} */ r#"NAME
    rm - remove files and directories

SYNOPSIS
    rm [-h | --help] [-d | --dir] [-i] [-r | -R | --recursive] [-v | --verbose] TARGETS...

DESCRIPTION
    Removes each specified file, but does not remove directories by default.

OPTIONS
    -h
    --help
        Display this help information and exit.

    -d
    --dir
        Remove empty directories in addition to files.

    -i
    --interactive
        Prompt before removing all files and directories.

    -r
    -R
    --recursive
        Remove directories and their contents recursively.

    -v
    --verbose
        Print the file changes that have been successfully performed.

AUTHOR
    Written by Michael Murphy.
"#; /* @MANEND */

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("i", "interactive")
        .add_flag("r", "recursive")
        .add_flag("R", "")
        .add_flag("d", "dir")
        .add_flag("v", "verbose")
        .add_flag("h", "help");
    parser.parse(env::args());

    if parser.found(&'h') || parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }
    if parser.found(&'r') || parser.found(&'R') || parser.found("recursive") {
        *parser.flag(&'d') = true;
        *parser.flag("directory") = true;
    }
    if let Err(err) = parser.found_invalid() {
        stderr.write(err.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
    }
    if parser.args.is_empty() {
        stdout.write(b"missing operand\nTry 'rm --help' for more information.\n").try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let mut exit_status = 0i32;
    for arg in &parser.args {
        if fs::metadata(&arg).is_err() {
            stderr.write(b"aborting due to invalid path: '").try(&mut stderr);
            stderr.write(arg.as_bytes()).try(&mut stderr);
            stderr.write(b"'\n").try(&mut stderr);
            stderr.flush().try(&mut stderr);
            exit(1);
        }
    }
    for arg in &parser.args {
        if Path::new(arg).is_dir() {
            if parser.found(&'i') || parser.found("interactive") {
                stdout.write(b"remove directory '").try(&mut stderr);
                stdout.write(arg.as_bytes()).try(&mut stderr);
                stdout.write(b"'? ").try(&mut stderr);
                stdout.flush().try(&mut stderr);
                let input = &mut String::new();
                let stdin = io::stdin();
                stdin.read_line(input).try(&mut stderr);
                if input.chars().next().unwrap() != 'y' { continue }
            }
            if parser.found(&'d') || parser.found("directory") {
                // Attempt to remove a directory and all of it's contents if recursive mode is enabled.
                // If recursion is not enabled, attempt to remove the directory if it is empty.
                if parser.found(&'r') || parser.found(&'R') || parser.found("recursive") {
                    // TODO: Use walkdir when it is implemented in Redox instead of fs::remove_dir_all().
                    if let Err(message) = fs::remove_dir_all(Path::new(arg)) {
                        stderr.write(b"cannot remove directory '").try(&mut stderr);
                        stderr.write(arg.as_bytes()).try(&mut stderr);
                        stderr.write(b"': ").try(&mut stderr);
                        print_error(message, &mut stderr);
                        exit_status = 1;
                    } else if parser.found(&'v') || parser.found("verbose") {
                        stdout.write(b"removed directory '").try(&mut stderr);
                        stdout.write(arg.as_bytes()).try(&mut stderr);
                        stdout.write(b"'\n").try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                    }
                } else {
                    if let Err(message) = fs::remove_dir(Path::new(arg)) {
                        stderr.write(b"cannot remove directory '").try(&mut stderr);
                        stderr.write(arg.as_bytes()).try(&mut stderr);
                        stderr.write(b"': ").try(&mut stderr);
                        print_error(message, &mut stderr);
                        exit_status = 1;
                    } else if parser.found(&'v') || parser.found("verbose") {
                        stdout.write(b"removed directory '").try(&mut stderr);
                        stdout.write(arg.as_bytes()).try(&mut stderr);
                        stdout.write(b"'\n").try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                    }
                }
            } else {
                stderr.write(b"cannot remove '").try(&mut stderr);
                stderr.write(arg.as_bytes()).try(&mut stderr);
                stderr.write(b"': is a directory\n").try(&mut stderr);
                stderr.flush().try(&mut stderr);
                exit_status = 1;
            }
        }
        else {
            // Attempt to remove the file given as an input argument.
            if parser.found(&'i') || parser.found("interactive") {
                stdout.write(b"remove file '").try(&mut stderr);
                stdout.write(arg.as_bytes()).try(&mut stderr);
                stdout.write(b"'? ").try(&mut stderr);
                stdout.flush().try(&mut stderr);
                let input = &mut String::new();
                let stdin = io::stdin();
                stdin.read_line(input).try(&mut stderr);
                if input.chars().next().unwrap() != 'y' { continue }
            }
            if let Err(message) = fs::remove_file(Path::new(arg)) {
                stderr.write(b"cannot remove '").try(&mut stderr);
                stderr.write(arg.as_bytes()).try(&mut stderr);
                stderr.write(b"': ").try(&mut stderr);
                print_error(message, &mut stderr);
                exit_status = 1;
            } else if parser.found(&'v') || parser.found("verbose") {
                stdout.write(b"removed '").try(&mut stderr);
                stdout.write(arg.as_bytes()).try(&mut stderr);
                stdout.write(b"'\n").try(&mut stderr);
                stdout.flush().try(&mut stderr);
            }
        }
    }
    exit(exit_status);
}

/// Print the message given by an io::Error to stderr.
fn print_error(message: io::Error, stderr: &mut Stderr) {
    stderr.write(message.description().as_bytes()).try(stderr);
    stderr.write(b"\n").try(stderr);
    stderr.flush().try(stderr);
}
