#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::str;
use std::fs::File;
use std::process::exit;
use std::io::{stdout, stderr, stdin, Error, Write, BufRead, BufReader, BufWriter};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const HELP_INFO:       &'static str = "Try ‘uniq --help’ for more information.\n";
const REQUIRES_OPTION: &'static str = "option requires an argument\n";
const MAN_PAGE: &'static str = r#"
NAME
    uniq - report or omit repeated lines

SYNOPSIS
    uniq [ -h | --help | -d | -u | -i ] [FILE]...

DESCRIPTION
    Filter adjacent matching lines of FILE(s) to standard output.

    With no FILE(s), read standard input.

OPTIONS
    -c
    --count
        precede each output line with a count of the number of times the line occurred in the input
    -d
    --repeated-lines
        only print duplicate lines, one for each group
    -i
    --ignore-case
        compare lines case-insensitively
    -s
    --skip-chars=N
        avoid comparing the first N characters
    -u
    --unique-lines
        only print unique lines
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn lines_from_stdin() -> Result<Vec<Vec<u8>>, Error> {
    let stdin = stdin();
    let mut lines = Vec::new();

    let f = BufReader::new(stdin.lock());
    for line in f.split(b'\n') {
        lines.push(line?);
    }
    Ok(lines)
}

fn lines_from_files(paths: &Vec<&String>) -> Result<Vec<Vec<u8>>, Error> {
    let mut lines = Vec::new();

    for path in paths {
        let f = BufReader::new(File::open(path)?);
        for line in f.split(b'\n') {
            lines.push(line?);
        }
    }
    Ok(lines)
}

fn eq_strings(left: &Vec<u8>, right: &Vec<u8>, skip_chars: usize, ignore_case: bool) -> bool {
    let left = if skip_chars >= left.len() { &[] } else { &left[skip_chars..] };
    let right = if skip_chars >= right.len() { &[] } else { &right[skip_chars..] };

    if ignore_case {
        left.len() == right.len() &&
        left
            .iter()
            .zip(right.iter())
            .all(|(&l, &r)|  l | 0x20 == r | 0x20)

    } else {
        left == right
    }
}

fn get_squashed_lines(lines: &Vec<Vec<u8>>, skip_chars: usize, ignore_case: bool) -> Vec<(usize, &Vec<u8>)> {
    let mut squashed =  Vec::new();
    let llen = lines.len();

    let mut r: usize = 0;

    while r < llen {
        let mut rnext: usize = r + 1;
        let mut count: usize = 1;

        while rnext < llen && eq_strings(&lines[r], &lines[rnext], skip_chars, ignore_case) {
            count += 1;
            rnext += 1;
        }

        squashed.push((count, &lines[r]));
        r += count;
    }

    squashed
}

fn unique_lines(lines: Vec<(usize, &Vec<u8>)>) -> Vec<(usize, &Vec<u8>)> {
   lines.into_iter()
       .filter(|&(k,_)| k == 1)
       .collect::<Vec<_>>()
}

fn repeated_lines(lines: Vec<(usize, &Vec<u8>)>) -> Vec<(usize, &Vec<u8>)> {
   lines.into_iter()
       .filter(|&(k,_)| k > 1)
       .collect::<Vec<_>>()
}

fn main() {

    let stdout = stdout();
    let mut stdout = BufWriter::with_capacity(8192, stdout.lock());
    let mut stderr = stderr();
    let mut parser = ArgParser::new(6)
        .add_opt("s", "skip-chars")
        .add_flag(&["c", "count"])
        .add_flag(&["d", "repeated-lines"])
        .add_flag(&["i", "ignore-case"])
        .add_flag(&["u", "unique-lines"])
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let lines = match parser.args.is_empty() {
        true => lines_from_stdin(),
        false => {
            let paths = parser.args.iter().collect::<Vec<_>>();
            lines_from_files(&paths)
        }
    };

    match lines {
        Ok(ref l) => {
            let mut skip_chars: usize = 0;
            if parser.found(&'s') || parser.found("skip-chars") {
                if let Some(c) = parser.get_opt(&'s') {
                    skip_chars = c.parse::<usize>().try(&mut stderr);
                } else if let Some(c) = parser.get_opt("skip-chars") {
                    skip_chars = c.parse::<usize>().try(&mut stderr);
                } else {
                    stderr.write_all(REQUIRES_OPTION.as_bytes()).try(&mut stderr);
                    stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                    stderr.flush().try(&mut stderr);
                    exit(1);
                }
            }

            let mut squashed = get_squashed_lines(&l, skip_chars, parser.found("ignore-case"));

            if parser.found("unique-lines") {
                squashed = unique_lines(squashed);
            } else if parser.found("repeated-lines") {
                squashed = repeated_lines(squashed);
            }

            if parser.found("count") {
                for (c, v) in squashed {
                    let _ = stdout.write_fmt(format_args!("{} ", c))
                        .and_then(|_| stdout.write_all(b" "))
                        .and_then(|_| stdout.write_all(v))
                        .and_then(|_| stdout.write_all(b"\n"));
                }
            } else {
                for (_, v) in squashed {
                    let _ = stdout.write_all(v)
                        .and_then(|_| stdout.write_all(b"\n"));
                }
            }
        }
        Err(e) => {
            let _ = write!(stderr, "{}", e);
        }
    }
}
