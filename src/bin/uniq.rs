#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::process::exit;
use std::io::{stdout, stderr, stdin, Error, Write, BufRead, BufReader};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = r#"
NAME
    uniq - report or omit repeated lines

SYNOPSIS
    uniq [ -h | --help | -d | -u | -i ] [FILE]...

DESCRIPTION
    Filter adjacent matching lines of FILE(s) to standard output.

    With no FILE(s), read standard input.

OPTIONS
    -h
    --help
        display this help and exit
    -c
    --count
        precede each output line with a count of the number of times the line occurred in the input
    -i
    --ignore-case
        compare lines case-insensitively
    -d
    --repeated-lines
        only print duplicate lines, one for each group
    -u
    --unique-lines
        only print unique lines
"#; /* @MANEND */

fn lines_from_stdin() -> Result<Vec<String>, Error> {
    let stdin = stdin();
    let mut lines = Vec::new();

    let f = BufReader::new(stdin.lock());
    for line in f.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

fn lines_from_files(paths: &Vec<&String>) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();

    for path in paths {
        let f = BufReader::new(File::open(path)?);
        for line in f.lines() {
            lines.push(line?);
        }
    }
    Ok(lines)
}

fn eq_strings(left: &str, right: &str, ignore_case: bool) -> bool {
    if ignore_case {
        left.to_lowercase() == right.to_lowercase()
    } else {
        left == right
    }
}

fn get_squashed_lines(lines: &[String], ignore_case: bool) -> Vec<(usize, &String)> {
    let mut squashed =  Vec::new();
    let llen = lines.len();

    let mut r: usize = 0;

    while r < llen {
        let mut rnext: usize = r + 1;
        let mut count: usize = 1;

        while rnext < llen && eq_strings(&lines[r], &lines[rnext], ignore_case) {
            count += 1;
            rnext += 1;
        }

        squashed.push((count, &lines[r]));
        r += count;
    }

    squashed
}

fn unique_lines(lines: Vec<(usize, &String)>) -> Vec<(usize, &String)> {
   lines.into_iter()
       .filter(|&(k,_)| k == 1)
       .collect::<Vec<_>>()
}

fn repeated_lines(lines: Vec<(usize, &String)>) -> Vec<(usize, &String)> {
   lines.into_iter()
       .filter(|&(k,_)| k > 1)
       .collect::<Vec<_>>()
}

fn main() {

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(5)
        .add_flag(&["i", "ignore-case"])
        .add_flag(&["c", "count"])
        .add_flag(&["d", "repeated-lines"])
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
            let mut squashed = get_squashed_lines(&l, parser.found("ignore-case"));

            if parser.found("unique-lines") {
                squashed = unique_lines(squashed);
            } else if parser.found("repeated-lines") {
                squashed = repeated_lines(squashed);
            }

            if parser.found("count") {
                for (c, v) in squashed {
                    println!("{} {}", c, v);
                }
            } else {
                for (_, v) in squashed {
                    println!("{}", v);
                }
            }
        }
        Err(e) => {
            let _ = write!(stderr, "{}", e);
        }
    }
}
