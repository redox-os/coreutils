#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use coreutils::extra::{OptionalExt, fail};

static MAN_PAGE: &'static str = r#"NAME
    head - output the first part of a file

SYNOPSIS
    head [[-h | --help] | [-n LINES] | [-c BYTES]] [FILE ...]

DESCRIPTION
    Print the first 10 lines of each FILE to standard output. If there are no files, read the standard input. If there are multiple files, prefix each one with a header containing it's name.

OPTIONS
    -h
    --help
        Print this manual page.

    -n [-]LINES
        Print the first LINES lines. If prefixed with a minus, print all but the last LINES lines.

    -c [-]BYTES
        Print the first BYTES bytes. If prefixed with a minus, print all but the last BYTES bytes.

AUTHOR
    Written by Žad Deljkić.
"#;

// lines - true if outputing lines, false if outputing bytes
// num - number of lines/bytes specified
// skip - false if outputing first num lines/bytes, true if outputing all but the last num lines/bytes (i.e. skip the last num lines/bytes)
#[derive(Copy, Clone)]
struct Options {
    lines: bool,
    num: usize,
    skip: bool,
}

// get the last line/byte up to which we read
// while taking care to stay within bounds
fn get_last(num: usize, len: usize, skip: bool) -> usize {
    if skip {
        if num <= len {
            len - num
        } else {
            0
        }
    } else {
        if num <= len {
            num
        } else {
            len
        }
    }
}

fn head<R: Read>(mut input: R, opts: Options) -> io::Result<()> {
    if opts.lines {
        let lines: Vec<String> = try!(io::BufReader::new(input).lines().collect());

        let last = get_last(opts.num, lines.len(), opts.skip);

        for line in &lines[..last] {
            println!("{}", line);
        }
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        let mut output = Vec::<u8>::new();
        try!(input.read_to_end(&mut output));

        let last = get_last(opts.num, output.len(), opts.skip);

        try!(stdout.write_all(&output[..last]));
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

        // check if number of lines/bytes is prefixed with a minus
        if args[1].starts_with("-") {
            opts.skip = true;
            args[1].remove(0); // remove the minus
        }

        opts.num = args[1].parse::<usize>().try(&mut stderr);

        // remove the arguments specifiyng the number of lines/bytes
        args = args.split_off(2);
    }

    // the rest of the arguments are now files
    if args.is_empty() {
        head(io::stdin(), opts).try(&mut stderr);
    } else if args.len() == 1 {
        let file = fs::File::open(&args[0]).try(&mut stderr);
        head(file, opts).try(&mut stderr);
    } else {
        for path in args {
            let file = fs::File::open(&path).try(&mut stderr);
            println!("==> {} <==", path);
            head(file, opts).try(&mut stderr);
            println!("");
        }
    }
}
