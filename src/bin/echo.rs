#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use arg_parser::ArgParser;
use extra::option::OptionalExt;

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
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(4)
        .add_flag(&["e", "escape"])
        .add_flag(&["n", "no-newline"])
        .add_flag(&["s", "no-spaces"])
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    // Print to standard output
    let mut first = true;
    for arg in parser.args.iter().map(|x| x.as_bytes()) {
        if first {
            first = false;
        } else if ! parser.found("no-spaces") {
            stdout.write(&[b' ']).try(&mut stderr);
        }
        if parser.found("escape") {
            let mut check = false;
            for &byte in arg {
                match byte {
                    b'\\' if check => {
                        stdout.write(&[byte]).try(&mut stderr);
                        check = false;
                    },
                    b'\\' => check = true,
                    b'a' if check => {
                        stdout.write(&[7u8]).try(&mut stderr); // bell
                        check = false;
                    },
                    b'b' if check => {
                        stdout.write(&[8u8]).try(&mut stderr); // backspace
                        check = false;
                    },
                    b'c' if check => {
                        exit(0);
                    },
                    b'e' if check => {
                        stdout.write(&[27u8]).try(&mut stderr); // escape
                        check = false;
                    },
                    b'f' if check => {
                        stdout.write(&[12u8]).try(&mut stderr); // form feed
                        check = false;
                    },
                    b'n' if check => {
                        stdout.write(&[b'\n']).try(&mut stderr); // newline
                        check = false;
                    },
                    b'r' if check => {
                        stdout.write(&[b'\r']).try(&mut stderr);
                        check = false;
                    },
                    b't' if check => {
                        stdout.write(&[b'\t']).try(&mut stderr);
                        check = false;
                    },
                    b'v' if check => {
                        stdout.write(&[11u8]).try(&mut stderr); // vertical tab
                        check = false;
                    },
                    _ if check => {
                        stdout.write(&[b'\\', byte]).try(&mut stderr);
                        check = false;
                    },
                    _ => { stdout.write(&[byte]).try(&mut stderr); }
                }
            }
        } else {
            stdout.write(arg).try(&mut stderr);
        }
    }

    if ! parser.found("no-newline") {
        stdout.write(&[b'\n']).try(&mut stderr);
    }
}
