#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::{WriteExt, fail};

static MAN_PAGE: &'static str = /* @MANSTART{tail} */ r#"
NAME
    tail - output the last part of a file

SYNOPSIS
    tail [[-h | --help] | [[-n | --lines] [+]LINES] | [[-c | --bytes] [+]BYTES]] [FILE ...]

DESCRIPTION
    Print the last 10 lines of each FILE to standard output. If there are no files, read the
    standard input. If there are multiple files, prefix each one with a header containing it's
    name.

OPTIONS
    -h
    --help
        Print this manual page.

    -n LINES
    --lines LINES
        Print the last LINES lines.

    -n +LINES
    --lines +LINES
        Print all but the first LINES lines.

    -c BYTES
    --bytes BYTES
        Print the last BYTES bytes.

    -c +BYTES
    --bytes +BYTES
        Print all but the first BYTES bytes.

AUTHOR
    Written by Žad Deljkić.
"#; /* @MANEND */

fn tail<R: Read, W: Write>(input: R, output: W, stderr: &mut Stderr, parser: &ArgParser) -> io::Result<()> {
    let mut writer = io::BufWriter::new(output);
    let (skip, num): (bool, usize) = 
        if let Some(num) = parser.get_opt(&'n') {
            (num.starts_with("+"), num.trim_left_matches('+').parse().try(stderr))
        }
        else if let Some(num) = parser.get_opt("lines") {
            (num.starts_with("+"), num.trim_left_matches('+').parse().try(stderr))
        }
        else if let Some(num) = parser.get_opt(&'c') {
            (num.starts_with("+"), num.trim_left_matches('+').parse().try(stderr))
        }
        else if let Some(num) = parser.get_opt("bytes") {
            (num.starts_with("+"), num.trim_left_matches('+').parse().try(stderr))
        }
        else {
            fail("missing argument (number of lines/bytes)", stderr);
        };

    if parser.flagged(&'n') || parser.flagged("lines") {
        if skip {
            let lines = io::BufReader::new(input).lines().skip(num);

            for line_res in lines {
                match line_res {
                    Ok(mut line) => {
                        line.push('\n');
                        try!(writer.write_all(line.as_bytes()));
                    }
                    Err(err) => return Err(err),
                };
            }
        } else {
            let lines = io::BufReader::new(input).lines();
            let mut deque = VecDeque::new();

            for line_res in lines {
                match line_res {
                    Ok(line) => {
                        deque.push_back(line);

                        if deque.len() > num {
                            deque.pop_front();
                        }
                    }
                    Err(err) => return Err(err),
                };
            }

            for mut line in deque {
                line.push('\n');
                try!(writer.write_all(line.as_bytes()));
            }
        }
    }
    else if parser.flagged(&'c') || parser.flagged("bytes") {
        if skip {
            let bytes = input.bytes().skip(num);

            for byte_res in bytes {
                match byte_res {
                    Ok(byte) => try!(writer.write_all(&[byte])),
                    Err(err) => return Err(err),
                };
            }
        }
        else {
            let bytes = input.bytes();
            let mut deque = VecDeque::new();

            for byte_res in bytes {
                match byte_res {
                    Ok(byte) => {
                        deque.push_back(byte);

                        if deque.len() > num {
                            deque.pop_front();
                        }
                    }
                    Err(err) => return Err(err),
                };
            }

            for byte in deque {
                try!(writer.write_all(&[byte]));
            }
        }
    }

    Ok(())
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(3)
        .add_opt_default("n", "lines", "10")
        .add_opt("c", "bytes")
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged(&'h') || parser.flagged("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }
    if parser.flagged(&'c') || parser.flagged("bytes") {
        parser.set_opt(&'n', None);
        parser.set_opt("lines", None);
    }
    if let Err(err) = parser.flagged_invalid() {
        stderr.write_all(err.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        return;
    }

    // run the main part
    if parser.args.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        tail(stdin, stdout, &mut stderr, &parser).try(&mut stderr);
    } else if parser.args.len() == 1 {
        let file = fs::File::open(&parser.args[0]).try(&mut stderr);
        tail(file, stdout, &mut stderr, &parser).try(&mut stderr);
    } else {
        for path in &parser.args {
            let file = fs::File::open(&path).try(&mut stderr);
            stdout.write(b"==> ").try(&mut stderr);
            stdout.write(path.as_bytes()).try(&mut stderr);
            stdout.writeln(b" <==").try(&mut stderr);
            tail(file, &mut stdout, &mut stderr, &parser).try(&mut stderr);
        }
    }
}
