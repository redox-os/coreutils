#![deny(warnings)]

extern crate extra;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Stderr, StdoutLock, Write};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{cat} */ r#"NAME
    cat - concatenate files and print on the standard output

SYNOPSIS
    cat [-h | --help] [-A | --show-all] [-b | --number-nonblank] [-e] [-E | --show-ends]
        [-n | --number] [-s | --squeeze-blank] [-t] [-T] FILES...

DESCRIPTION
    Concatenates all files to the standard output.

    If no file is given, or if FILE is '-', read from standard input.

OPTIONS
    -A
    --show-all
        equivalent to -v -E -T

    -b
    --number-nonblank
        number nonempty output lines, overriding -n

    -e
        equivalent to -v -E

    -E
    --show-ends
        display $ at the end of each line

    -n
    --number
        number all output lines

    -s
    --squeeze-blank
        supress repeated empty output lines

    -t
        equivalent to -v -T

    -T
    --show_tabs
        display TAB characters as ^I

    -v
    --show-nonprinting
        use ^ and M- notation,except for LFD and TAB.

    -h
    --help
        display this help and exit

AUTHOR
    Written by Michael Murphy.
"#; /* @MANEND */

struct Program {
    number:          bool,
    number_nonblank: bool,
    show_ends:       bool,
    show_tabs:       bool,
    squeeze_blank:   bool,
    paths:           Vec<String>,
}

impl Program {
    /// Initialize the program's arguments and flags.
    fn initialize(stdout: &mut StdoutLock, stderr: &mut Stderr) -> Program {
        let mut cat = Program {
            number:          false,
            number_nonblank: false,
            show_ends:       false,
            show_tabs:       false,
            squeeze_blank:   false,
            paths:           Vec::new()
        };
        for arg in env::args().skip(1) {
            if arg.starts_with('-') {
                match arg.as_str() {
                    "-h" | "--help" => {
                        stdout.write(MAN_PAGE.as_bytes()).try(stderr);
                        stdout.flush().try(stderr);
                        std::process::exit(0);
                    }
                    "-A" | "--show-all" => {
                        // TODO: implement -v
                        cat.show_ends = true;
                        cat.show_tabs = true;
                    },
                    "-b" | "--number-nonblank" => {
                        cat.number_nonblank = true;
                        cat.number = false;
                    },
                    "-e" => {
                        // TODO: implement -v
                        cat.show_ends = true;
                    },
                    "-E" | "--show-ends" => cat.show_ends = true,
                    "-n" | "--number" => {
                        cat.number = true;
                        cat.number_nonblank = false;
                    },
                    "-s" | "--squeeze-blank" => cat.squeeze_blank = true,
                    "-t" => {
                        // TODO: implement -v
                        cat.show_tabs = true;
                    },
                    "-T" => cat.show_tabs = true,
                    "-v" | "--show-nonprinting" => {
                        // TODO: implement -v, --show-nonprinting
                        continue
                    },
                    _ => {
                        stderr.write(b"invalid option -- '").try(stderr);
                        stderr.write(arg.as_bytes()).try(stderr);
                        stderr.write(b"'\nTry 'cat --help' for more information.\n").try(stderr);
                        stderr.flush().try(stderr);
                        std::process::exit(1);
                    }
                }
            } else {
                cat.paths.push(arg);
            }
        }
        cat
    }

    /// Execute the parameters given to the program.
    fn execute(&self, stdout: &mut StdoutLock, stderr: &mut Stderr) {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut exit_status = 0i32;

        if self.paths.len() == 0 {
            io::copy(&mut stdin, stdout).try(stderr);
        } else {
            let mut line_count = 1;
            for path in &self.paths {
                if self.number || self.number_nonblank || self.show_ends || self.show_tabs || self.squeeze_blank {
                    let file = match fs::File::open(&path) {
                        Ok(file) => file,
                        Err(message) => {
                            stderr.write(&path.as_bytes()).try(stderr);
                            stderr.write(b": ").try(stderr);
                            stderr.write(message.description().as_bytes()).try(stderr);
                            stderr.write(b"\n").try(stderr);
                            stderr.flush().try(stderr);
                            exit_status = 1;
                            continue
                        }
                    };
                    let mut current_line: Vec<u8> = Vec::new();
                    let mut last_line_was_blank = false;
                    for byte in file.bytes().map(|x| x.unwrap_or(b' ')) {
                        if byte == b'\n' {
                            if current_line.is_empty() {
                                if last_line_was_blank && self.squeeze_blank {
                                    continue
                                } else if !last_line_was_blank {
                                    last_line_was_blank = true;
                                }
                            }
                            if self.number || (self.number_nonblank && !current_line.is_empty()) {
                                stdout.write(format!("     {}  ", line_count).as_bytes()).try(stderr);
                                line_count += 1;
                            }
                            stdout.write(current_line.as_slice()).try(stderr);
                            if self.show_ends { stdout.write(b"$\n").try(stderr); } else { stdout.write(b"\n").try(stderr); }
                            stdout.flush().try(stderr);
                            current_line.clear();
                        } else {
                            if self.show_tabs && byte == b'\t' {
                                current_line.push(b'^');
                                current_line.push(b'I');
                            } else {
                                current_line.push(byte);
                            }
                        }
                    }
                } else {
                    if path == "-" {
                        io::copy(&mut stdin, stdout).try(stderr);
                    } else {
                        let file = fs::File::open(&path);
                        let mut file = file.try(stderr);

                        io::copy(&mut file, stdout).try(stderr);
                    }
                }
            }
        }
        std::process::exit(exit_status);
    }
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    Program::initialize(&mut stdout, &mut stderr).execute(&mut stdout, &mut stderr);
}
