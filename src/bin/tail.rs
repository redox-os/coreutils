#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use coreutils::extra::{OptionalExt, fail};

static MAN_PAGE: &'static str = r#"NAME
    tail - output the last part of a file

SYNOPSIS
    tail [[-h | --help] | [-n LINES] | [-c BYTES]] [FILE ...]

DESCRIPTION
    Print the last 10 lines of each FILE to standard output. If there are no files, read the standard input. If there are multiple files, prefix each one with a header containing it's name.

OPTIONS
    -h
    --help
        Print this manual page.

    -n [+]LINES
        Print the last LINES lines. If prefixed with a plus, print all but the first LINES lines.

    -c [+]BYTES
        Print the last BYTES bytes. If prefixed with a plus, print all but the first BYTES bytes.

AUTHOR
    Written by Žad Deljkić.
"#;

// lines - true if outputing lines, false if outputing bytes
// num - number of lines/bytes specified
// skip - false if outputing last num lines/bytes, true if outputing all but the first num lines/bytes (i.e. skip the first num lines/bytes)
#[derive(Copy, Clone)]
struct Options {
    lines: bool,
    num: usize,
    skip: bool,
}

// get the first line/byte from which we read
// while taking care to stay within bounds
fn get_first(num: usize, len: usize, skip: bool) -> usize {
    if skip {
        if num <= len {
            num
        } else {
            len
        }
    } else {
        if num <= len {
            len - num
        } else {
            0
        }
    }
}

fn tail<R: Read>(mut input: R, opts: Options) -> io::Result<()> {
    if opts.lines {
        let lines: Vec<String> = try!(io::BufReader::new(input).lines().collect());

        let first = get_first(opts.num, lines.len(), opts.skip);

        for line in &lines[first..] {
            println!("{}", line);
        }
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        let mut output = Vec::<u8>::new();
        try!(input.read_to_end(&mut output));

        let first = get_first(opts.num, output.len(), opts.skip);

        try!(stdout.write_all(&output[first..]));
    }

    Ok(())
}

fn main() {
    // default options
    let mut opts = Options {
        lines: true,
        num: 10,
        skip: false,
    };
    let mut stderr = io::stderr();
    let mut args: Vec<_> = env::args().skip(1).collect();

    // parse options
    if args.len() > 0 && args[0].starts_with("-") {
        match args[0].as_str() {
            "-h" | "--help" => {
                print!("{}", MAN_PAGE);
                return;
            }
            "-n" => opts.lines = true,
            "-c" => opts.lines = false,
            _ => fail("invalid option", &mut stderr),
        }

        if args.len() < 2 {
            fail("missing parameter (number of lines/bytes)", &mut stderr);
        }

        // check if number of lines/bytes is prefixed with a plus
        if args[1].starts_with("+") {
            opts.skip = true;
        }

        opts.num = args[1].parse::<usize>().try(&mut stderr);

        // remove the arguments specifiyng the number of lines/bytes
        args = args.split_off(2);
    }

    // the rest of the arguments are now files
    if args.is_empty() {
        tail(io::stdin(), opts).try(&mut stderr);
    } else if args.len() == 1 {
        let file = fs::File::open(&args[0]).try(&mut stderr);
        tail(file, opts).try(&mut stderr);
    } else {
        for path in args {
            let file = fs::File::open(&path).try(&mut stderr);
            println!("==> {} <==", path);
            tail(file, opts).try(&mut stderr);
            println!("");
        }
    }
}
