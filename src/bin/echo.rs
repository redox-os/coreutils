#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use extra::option::OptionalExt;
use std::process::exit;

const MAN_PAGE: &'static str = /* @MANSTART{echo} */ r#"
NAME
    echo - display a line of text

SYNOPSIS
    echo [ -h | --help ] [-e] [-n] [-s] [STRING]...

DESCRIPTION
    Print the STRING(s) to standard output.

OPTIONS
    -e
        enable the interpretation of backslash escapes
    -n
        do not output the trailing newline
    -s
        do not separate arguments with spaces

    Escape Sequences
        When the -e argument is used, the following sequences will be interpreted:

        \\  backslash

        \a  alert (BEL)

        \b  backspace (BS)

        \c  produce no further output

        \e  escape (ESC)

        \f  form feed (FF)

        \n  new line

        \r  carriage return

        \t  horizontal tab (HT)

        \v  vertical tab (VT)

"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let stdout = &mut stdout.lock();
    let stderr = &mut stderr();
    let args = env::args().skip(1).collect::<Vec<String>>();

    let (mut no_newline, mut no_spaces, mut escape) = (false, false, false);

    // Check for specific flags
    for argument in args.iter().map(|x| x.as_bytes()) {
        if argument.len() == 2 && argument[0] == b'-' {
            match argument[1] {
                b'h' => {
                    stdout.write(MAN_PAGE.as_bytes()).try(stderr);
                    stdout.flush().try(stderr);
                    exit(0);
                }
                b'n' => no_newline = true,
                b's' => no_spaces = true,
                b'e' => escape = true,
                _   => ()
            }
        }
    }

    // Print to standard output
    for argument in args.iter().map(|x| x.as_bytes()) {
        if argument.len() == 2 && argument[0] == b'-' {
            match argument[1] {
                b'n' | b's' | b'e' => continue,
                _ => { stdout.write(argument).try(stderr); }
            }
        } else {
            if escape {
                let mut check = false;
                for &byte in argument {
                    match byte {
                        b'\\' if check => {
                            stdout.write(&[byte]).try(stderr);
                            check = false;
                        },
                        b'\\' => check = true,
                        b'a' if check => {
                            stdout.write(&[7u8]).try(stderr); // bell
                            check = false;
                        },
                        b'b' if check => {
                            stdout.write(&[8u8]).try(stderr); // backspace
                            check = false;
                        },
                        b'c' if check => {
                            exit(0);
                        },
                        b'e' if check => {
                            stdout.write(&[27u8]).try(stderr); // escape
                            check = false;
                        },
                        b'f' if check => {
                            stdout.write(&[12u8]).try(stderr); // form feed
                            check = false;
                        },
                        b'n' if check => {
                            stdout.write(&[b'\n']).try(stderr); // newline
                            check = false;
                        },
                        b'r' if check => {
                            stdout.write(&[b'\r']).try(stderr);
                            check = false;
                        },
                        b't' if check => {
                            stdout.write(&[b'\t']).try(stderr);
                            check = false;
                        },
                        b'v' if check => {
                            stdout.write(&[11u8]).try(stderr); // vertical tab
                            check = false;
                        },
                        _ if check => {
                            stdout.write(&[b'\\', byte]).try(stderr);
                            check = false;
                        },
                        _ => { stdout.write(&[byte]).try(stderr); }
                    }
                }
            } else {
                stdout.write(argument).try(stderr);
            }
        }
        if !no_spaces { stdout.write(&[b' ']).try(stderr); }
    }

    if !no_newline { stdout.write(&[b'\n']).try(stderr); }
}
