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
        let line = match line {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        lines.push(line);
    }
    Ok(lines)
}

fn lines_from_files(paths: &Vec<&String>) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();

    for path in paths {
        let f = try!(File::open(path));
        let f = BufReader::new(f);
        for line in f.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => return Err(e),
            };
            lines.push(line);
        }
    }
    Ok(lines)
}

fn repeated_lines(vector: Vec<String>) -> Vec<String> {
    let mut repeated_lines =  Vec::new();

    let ln = vector.len();
    if ln <= 1 {
       return repeated_lines;
    }

    let mut r: usize = 1;

    while r < ln {
        let first = &vector[(r - 1)];
        let second = &vector[r];
        if first == second {
            repeated_lines.push(first.to_owned());
            r += 2;  // skip repeatead
        } else {
            r += 1;
        }
        if r == ln {
            let last = &vector[ln - 1];
            repeated_lines.push(last.to_owned());
        }
    }
    repeated_lines
}

fn unique_lines(vector: Vec<String>) -> Vec<String> {
    let mut unique_lines =  Vec::new();

    let ln = vector.len();
    if ln <= 1 {
       return vector;
    }

    let mut r: usize = 1;

    while r < ln {
        let first = &vector[(r - 1)];
        let second = &vector[r];
        if first != second {
            unique_lines.push(first.to_owned());
            r += 1;
        } else {
            r += 2;  // skip repeatead
        }
        if r == ln {
            let last = &vector[ln - 1];
            unique_lines.push(last.to_owned());
        }
    }
    unique_lines
}

fn main() {

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(2)
        .add_flag(&["i", "ignore-case"])
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
            let mut paths = Vec::new();
            for dir in parser.args.iter() {
                paths.push(dir);
            }
            lines_from_files(&paths)
        }
    };

    match lines {
        Ok(mut l) => {
            if parser.found("ignore-case") {
                l = l.iter().map(|x| x.to_lowercase()).collect();
            }
            if parser.found("unique-lines") {
                l = unique_lines(l);
            } else if parser.found("repeated-lines") {
                l = repeated_lines(l);
            } else {
                l.dedup();
            }
            for x in l {
                println!("{}", x);
            }
        }
        Err(e) => {
            let _ = write!(stderr, "{}", e);
        }
    }
}
