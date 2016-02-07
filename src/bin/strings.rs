// If this code works, it was written by Ticki. If it does not, I don't know who the hell wrote it
// but it was definitively not me. Blame someone else.

#[deny(warnings)]
#[deny(unused_mut)]

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write, Read};
use std::process::exit;

#[derive(Copy, Clone)]
struct Trailing {
    chars: [u8; 4],
    current: usize,
}

// Wow, such premature, much optimization
impl Trailing {
    #[inline]
    fn new() -> Trailing {
        Trailing {
            chars: [0; 4],
            current: 0,
        }
    }

    fn set(&mut self, b: u8) -> bool {
        self.chars[self.current] = b;
        self.current += 1;

        self.is_complete()
    }

    fn reset(&mut self) {
        self.current = 0;
    }

    fn is_complete(self) -> bool {
        self.current == 4
    }

    fn chars(self) -> [u8; 4] {
        self.chars
    }
}

trait IsPrintable {
    fn is_printable(self) -> bool;
}

impl IsPrintable for u8 {
    fn is_printable(self) -> bool {
        // TODO handle unicode.
        self >= 0x20 && self <= 0x7e
    }
}

trait Try {
    type Succ;

    fn try(self) -> Self::Succ;
}

impl<T, U: Error> Try for Result<T, U> {
    type Succ = T;

    fn try(self) -> T {
        match self {
            Ok(succ) => succ,
            Err(e) => {
                println!("error: {}", e.description());
                exit(1);
            },
        }
    }
}

fn read<R: Read, W: Write>(stdin: R, mut stdout: W) {
    let mut trailing = Trailing::new();

    for i in stdin.bytes().map(|x| x.try()) {
        if i.is_printable() {
            if trailing.is_complete() {
                stdout.write(&[i]).try();
            } else if trailing.set(i) {
                stdout.write(&trailing.chars()).try();
            }
        } else {
            if trailing.is_complete() {
                stdout.write(&[b'\n']).try();
            }
            trailing.reset();
        }
    }
}

const HELP: &'static [u8] = br#"
    NAME
        strings - inspect a binary file for strings of printable characters.
    SYNOPSIS
        strings [-h | --help] [FILE]
    DESCRIPTION
        This utility will read the file from the path given in the argument. If no argument is given, 'strings' will read from the standard input. The byte stream is then inspected for contiguous, printable ASCII characters of length 4 or more. These strings of printable characters are written to the standard output. Each contiguous strings are seperated by a newline (0x0A).

        This utility is useful for inspecting binary files for human readable information, to determine the contents.

        This is a clone of GNU strings, though they differ in a number of ways.
    OPTIONS
        -h
        --help
            Print this manual page.
    COPYRIGHT
        Copyright (c) 2016 Ticki

        Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

        The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#;

fn main() {
    let mut stdout = io::stdout();
    let mut args = env::args();
    if args.len() > 2 {
        println!("error: Too many arguments. Try 'strings -h'.");
        exit(1);
    }

    match args.nth(1) {
        None => read(io::stdin(), stdout),
        Some(a) => match a.as_ref() {
            "-h" | "--help" => {
                stdout.write(HELP).try();
            },
            f => read(fs::File::open(f).try(), stdout),
        },
    }
}
