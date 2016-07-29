#![deny(warnings)]
extern crate extra;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write, Stderr, Stdout, StdoutLock};
use std::path::{Path, PathBuf};
use std::process::exit;
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
    let mut rm = Program::initialize(&stdout);
    rm.execute();
}

struct Program<'a> {
    arguments:   Vec<PathBuf>,
    directory:   bool,
    interactive: bool,
    recursive:   bool,
    verbose:     bool,
    stderr:      Stderr,
    stdout:      StdoutLock<'a>,
}

impl<'a> Program<'a> {
    /// Initialize the program's arguments and check if the paths are valid.
    fn initialize(stdout: &'a Stdout) -> Program<'a> {
        let mut rm = Program {
            arguments:   Vec::new(),
            directory:   false,
            interactive: false,
            recursive:   false,
            verbose:     false,
            stderr:      io::stderr(),
            stdout:      stdout.lock()
        };
        for argument in env::args().skip(1).collect::<Vec<String>>() {
            if argument.starts_with("--") {
                match argument.as_str() {
                    "--dir" => rm.directory = true,
                    "--help" => {
                        rm.stdout.write(MAN_PAGE.as_bytes()).try(&mut rm.stderr);
                        rm.stdout.flush().try(&mut rm.stderr);
                        exit(0);
                    },
                    "--recursive" => {
                        rm.directory = true;
                        rm.recursive = true;
                    },
                    "--verbose" => rm.verbose = true,
                    _ => {
                        rm.stderr.write(b"invalid argument '").try(&mut rm.stderr);
                        rm.stderr.write(argument.as_bytes()).try(&mut rm.stderr);
                        rm.stderr.write(b"'\n").try(&mut rm.stderr);
                        rm.stderr.flush().try(&mut rm.stderr);
                    }
                }
            } else if argument.starts_with("-") {
                for character in argument.chars().skip(1) {
                    match character {
                        'd' => rm.directory = true,
                        'h' => {
                            rm.stdout.write(MAN_PAGE.as_bytes()).try(&mut rm.stderr);
                            rm.stdout.flush().try(&mut rm.stderr);
                            exit(0);
                        },
                        'i' => rm.interactive = true,
                        'r' | 'R' => {
                            rm.directory = true;
                            rm.recursive = true;
                        },
                        'v'=> rm.verbose = true,
                        _ => {
                            rm.stderr.write(b"invalid argument '").try(&mut rm.stderr);
                            rm.stderr.write(&[character as u8]).try(&mut rm.stderr);
                            rm.stderr.write(b"'\n").try(&mut rm.stderr);
                            rm.stderr.flush().try(&mut rm.stderr);
                        }
                    }
                }
            } else {
                if fs::metadata(&argument).is_err() {
                    rm.stderr.write(b"aborting due to invalid path: '").try(&mut rm.stderr);
                    rm.stderr.write(&argument.as_bytes()).try(&mut rm.stderr);
                    rm.stderr.write(b"'\n").try(&mut rm.stderr);
                    rm.stderr.flush().try(&mut rm.stderr);
                    exit(1);
                } else {
                    rm.arguments.push(PathBuf::from(&argument));
                }
            }
        }
        if rm.arguments.len() == 0 {
            rm.stdout.write(b"missing operand\nTry 'rm --help' for more information.\n").try(&mut rm.stderr);
            rm.stdout.flush().try(&mut rm.stderr);
            exit(0);
        }
        rm
    }

    /// Executes the rm program's arguments using any given flags.
    fn execute(&mut self) {
        let mut exit_status = 0i32;
        for argument in self.arguments.clone() {
            if argument.is_dir() {
                if self.interactive {
                    self.stdout.write(b"remove directory '").try(&mut self.stderr);
                    self.stdout.write(argument.to_string_lossy().as_bytes()).try(&mut self.stderr);
                    self.stdout.write(b"'? ").try(&mut self.stderr);
                    self.stdout.flush().try(&mut self.stderr);
                    let input = &mut String::new();
                    let stdin = io::stdin();
                    stdin.read_line(input).try(&mut self.stderr);
                    if input.chars().next().unwrap() != 'y' { continue }
                }
                if self.directory {
                    let status = self.remove_directory(&argument);
                    if exit_status == 0 { exit_status = status; }
                } else {
                    self.stderr.write(b"cannot remove '").try(&mut self.stderr);
                    self.stderr.write(argument.to_string_lossy().as_bytes()).try(&mut self.stderr);
                    self.stderr.write(b"': is a directory\n").try(&mut self.stderr);
                    self.stderr.flush().try(&mut self.stderr);
                    exit_status = 1;
                }
            } else {
                if self.interactive {
                    self.stdout.write(b"remove file '").try(&mut self.stderr);
                    self.stdout.write(argument.to_string_lossy().as_bytes()).try(&mut self.stderr);
                    self.stdout.write(b"'? ").try(&mut self.stderr);
                    self.stdout.flush().try(&mut self.stderr);
                    let input = &mut String::new();
                    let stdin = io::stdin();
                    stdin.read_line(input).try(&mut self.stderr);
                    if input.chars().next().unwrap() != 'y' { continue }
                }
                let status = self.remove_file(&argument);
                if exit_status == 0 { exit_status = status; }
            }
        }
        exit(exit_status);
    }

    /// Attempt to remove the file given as an input argument.
    fn remove_file(&mut self, file: &Path) -> i32 {
        if let Err(message) = fs::remove_file(file) {
            self.stderr.write(b"cannot remove '").try(&mut self.stderr);
            self.stderr.write(file.to_string_lossy().as_bytes()).try(&mut self.stderr);
            self.stderr.write(b"': ").try(&mut self.stderr);
            print_error(message, &mut self.stderr);
            return 1i32;
        } else if self.verbose {
            self.stdout.write(b"removed '").try(&mut self.stderr);
            self.stdout.write(file.to_string_lossy().as_bytes()).try(&mut self.stderr);
            self.stdout.write(b"'\n").try(&mut self.stderr);
            self.stdout.flush().try(&mut self.stderr);
        }
        return 0i32;
    }

    /// Attempt to remove a directory and all of it's contents if recursive mode is enabled.
    /// If recursion is not enabled, attempt to remove the directory if it is empty.
    fn remove_directory(&mut self, directory: &Path) -> i32 {
        // TODO: Use walkdir when it is implemented in Redox instead of fs::remove_dir_all().
        if self.recursive {
            if let Err(message) = fs::remove_dir_all(directory) {
                self.stderr.write(b"cannot remove directory '").try(&mut self.stderr);
                self.stderr.write(directory.to_string_lossy().as_bytes()).try(&mut self.stderr);
                self.stderr.write(b"': ").try(&mut self.stderr);
                print_error(message, &mut self.stderr);
                return 1i32;
            } else if self.verbose {
                self.stdout.write(b"removed directory '").try(&mut self.stderr);
                self.stdout.write(directory.to_string_lossy().as_bytes()).try(&mut self.stderr);
                self.stdout.write(b"'\n").try(&mut self.stderr);
                self.stdout.flush().try(&mut self.stderr);
            }
            return 0i32;
        } else {
            if let Err(message) = fs::remove_dir(directory) {
                self.stderr.write(b"cannot remove directory '").try(&mut self.stderr);
                self.stderr.write(directory.to_string_lossy().as_bytes()).try(&mut self.stderr);
                self.stderr.write(b"': ").try(&mut self.stderr);
                print_error(message, &mut self.stderr);
                return 1i32;
            } else if self.verbose {
                self.stdout.write(b"removed directory '").try(&mut self.stderr);
                self.stdout.write(directory.to_string_lossy().as_bytes()).try(&mut self.stderr);
                self.stdout.write(b"'\n").try(&mut self.stderr);
                self.stdout.flush().try(&mut self.stderr);
            }
            return 0i32;
        }
    }
}

/// Print the message given by an io::Error to stderr.
fn print_error(message: io::Error, stderr: &mut Stderr) {
    stderr.write(message.description().as_bytes()).try(stderr);
    stderr.write(b"\n").try(stderr);
    stderr.flush().try(stderr);
}
