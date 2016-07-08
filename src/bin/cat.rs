#![deny(warnings)]

extern crate extra;

use std::cell::Cell; // Provide mutable fields in immutable structs
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, BufReader, Read, Stderr, StdoutLock, Write};
use std::process::exit;
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
        equivalent to -vET

    -b
    --number-nonblank
        number nonempty output lines, overriding -n

    -e
        equivalent to -vE

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
        equivalent to -vT

    -T
    --show_tabs
        display TAB characters as ^I

    -v
    --show-nonprinting
        use caret (^) and M- notation, except for LFD and TAB.

    -h
    --help
        display this help and exit

AUTHOR
    Written by Michael Murphy.
"#; /* @MANEND */

struct Program {
    exit_status:      Cell<i32>,
    number:           bool,
    number_nonblank:  bool,
    show_ends:        bool,
    show_tabs:        bool,
    show_nonprinting: bool,
    squeeze_blank:    bool,
    paths:            Vec<String>,
}

impl Program {
    /// Initialize the program's arguments and flags.
    fn initialize(stdout: &mut StdoutLock, stderr: &mut Stderr) -> Program {
        let mut cat = Program {
            exit_status:      Cell::new(0i32),
            number:           false,
            number_nonblank:  false,
            show_ends:        false,
            show_tabs:        false,
            show_nonprinting: false,
            squeeze_blank:    false,
            paths:            Vec::new()
        };
        for argument in env::args().skip(1) {
            if argument.starts_with('-') && &argument != "-" {
                if argument.starts_with("--") {
                    match argument.as_str() {
                        "--help" => {
                            stdout.write(MAN_PAGE.as_bytes()).try(stderr);
                            stdout.flush().try(stderr);
                            exit(0);
                        }
                        "--show-all" => {
                            cat.show_nonprinting = true;
                            cat.show_ends = true;
                            cat.show_tabs = true;
                        },
                        "--number-nonblank" => {
                            cat.number_nonblank = true;
                            cat.number = false;
                        },
                        "--show-ends" => cat.show_ends = true,
                        "--number" => {
                            cat.number = true;
                            cat.number_nonblank = false;
                        },
                        "--squeeze-blank" => cat.squeeze_blank = true,
                        "--show-tabs" => cat.show_tabs = true,
                        "--show-nonprinting" => {
                            cat.show_nonprinting = true;
                        },
                        _ => {
                            stderr.write(b"invalid option -- '").try(stderr);
                            stderr.write(argument.as_bytes()).try(stderr);
                            stderr.write(b"'\nTry 'cat --help' for more information.\n").try(stderr);
                            stderr.flush().try(stderr);
                            exit(1);
                        }
                    }
                } else {
                    for character in argument.bytes().skip(1) {
                        match character {
                            b'h' => {
                                stdout.write(MAN_PAGE.as_bytes()).try(stderr);
                                stdout.flush().try(stderr);
                                exit(0);
                            }
                            b'A' => {
                                cat.show_nonprinting = true;
                                cat.show_ends = true;
                                cat.show_tabs = true;
                            },
                            b'b'=> {
                                cat.number_nonblank = true;
                                cat.number = false;
                            },
                            b'e' => {
                                cat.show_nonprinting = true;
                                cat.show_ends = true;
                            },
                            b'E'=> cat.show_ends = true,
                            b'n' => {
                                cat.number = true;
                                cat.number_nonblank = false;
                            },
                            b's' => cat.squeeze_blank = true,
                            b't' => {
                                cat.show_nonprinting = true;
                                cat.show_tabs = true;
                            },
                            b'T' => cat.show_tabs = true,
                            b'v' => {
                                cat.show_nonprinting = true;
                            },
                            _ => {
                                stderr.write(b"invalid option -- '").try(stderr);
                                stderr.write(&[character]).try(stderr);
                                stderr.write(b"'\nTry 'cat --help' for more information.\n").try(stderr);
                                stderr.flush().try(stderr);
                                exit(1);
                            }
                        }
                    }
                }
            } else {
                cat.paths.push(argument);
            }
        }
        cat
    }

    /// Execute the parameters given to the program.
    fn and_execute(&self, stdout: &mut StdoutLock, stderr: &mut Stderr) -> i32 {
        let stdin = io::stdin();
        let line_count = &mut 0usize;
        let flags_enabled = self.number || self.number_nonblank || self.show_ends || self.show_tabs ||
                            self.squeeze_blank || self.show_nonprinting;

        if self.paths.is_empty() && flags_enabled {
            self.cat(&mut stdin.lock(), line_count, stdout, stderr);
        } else if self.paths.is_empty() {
            io::copy(&mut stdin.lock(), stdout).try(stderr);
        } else {
            for path in &self.paths {
                if flags_enabled && path == "-" {
                    self.cat(&mut stdin.lock(), line_count, stdout, stderr);
                } else if flags_enabled {
                    fs::File::open(&path)
                        // Open the file and copy the file's contents to standard output based input arguments.
                        .map(|file| self.cat(BufReader::new(file), line_count, stdout, stderr))
                        // If an error occurred, print the error and set the exit status.
                        .unwrap_or_else(|message| {
                            stderr.write(path.as_bytes()).try(stderr);
                            stderr.write(b": ").try(stderr);
                            stderr.write(message.description().as_bytes()).try(stderr);
                            stderr.write(b"\n").try(stderr);
                            stderr.flush().try(stderr);
                            self.exit_status.set(1i32);
                        });
                } else if path == "-" {
                    // Copy the standard input directly to the standard output.
                    io::copy(&mut stdin.lock(), stdout).try(stderr);
                } else {
                    // Open a file and copy the contents directly to standard output.
                    fs::File::open(&path).map(|ref mut file| { io::copy(file, stdout).try(stderr); })
                        // If an error occurs, print the error and set the exit status.
                        .unwrap_or_else(|message| {
                            stderr.write(path.as_bytes()).try(stderr);
                            stderr.write(b": ").try(stderr);
                            stderr.write(message.description().as_bytes()).try(stderr);
                            stderr.write(b"\n").try(stderr);
                            stderr.flush().try(stderr);
                            self.exit_status.set(1i32);
                        });
                }
            }
        }
        self.exit_status.get()
    }

    /// Cats either a file or stdin based on the flag arguments given to the program.
    fn cat<F: Read>(&self, file: F, line_count: &mut usize, stdout: &mut StdoutLock, stderr: &mut Stderr) {
        let mut character_count = 0;
        let mut last_line_was_blank = false;

        for byte in file.bytes().map(|x| x.unwrap()) {
            if (self.number && character_count == 0) || (character_count == 0 && self.number_nonblank && byte != b'\n') {
                stdout.write(b"     ").try(stderr);
                stdout.write(line_count.to_string().as_bytes()).try(stderr);
                stdout.write(b"  ").try(stderr);
                *line_count += 1;
            }
            match byte {
                0...8 | 11...31 => if self.show_nonprinting {
                    push_caret(stdout, stderr, byte+64);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                },
                9 => {
                    if self.show_tabs {
                        push_caret(stdout, stderr, b'I');
                    } else {
                        stdout.write(&[byte]).try(stderr);
                    }
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                }
                10 => {
                    if character_count == 0 {
                        if self.squeeze_blank && last_line_was_blank {
                            continue
                        } else if !last_line_was_blank {
                            last_line_was_blank = true;
                        }
                    } else {
                        last_line_was_blank = false;
                        character_count = 0;
                    }
                    if self.show_ends {
                        stdout.write(b"$\n").try(stderr);
                    } else {
                        stdout.write(b"\n").try(stderr);
                    }
                    stdout.flush().try(stderr);
                },
                32...126 => {
                    stdout.write(&[byte]).try(stderr);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                },
                127 => if self.show_nonprinting {
                    push_caret(stdout, stderr, b'?');
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                },
                128...159 => if self.show_nonprinting {
                    stdout.write(b"M-^").try(stderr);
                    stdout.write(&[byte-64]).try(stderr);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                } else {
                    stdout.write(&[byte]).try(stderr);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                },
                _ => if self.show_nonprinting {
                    stdout.write(b"M-").try(stderr);
                    stdout.write(&[byte-128]).try(stderr);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                } else {
                    stdout.write(&[byte]).try(stderr);
                    count_character(&mut character_count, &self.number, &self.number_nonblank);
                },
            }
        }
    }
}
/// Increase the character count by one if number printing is enabled.
fn count_character(character_count: &mut usize, number: &bool, number_nonblank: &bool) {
    if *number || *number_nonblank {
        *character_count += 1;
    }
}

/// Print a caret notation to stdout.
fn push_caret(stdout: &mut StdoutLock, stderr: &mut Stderr, notation: u8) {
    stdout.write(&[b'^']).try(stderr);
    stdout.write(&[notation]).try(stderr);
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    exit(Program::initialize(&mut stdout, &mut stderr).and_execute(&mut stdout, &mut stderr));
}
