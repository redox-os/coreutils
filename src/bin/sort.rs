#![deny(warnings)]

extern crate arg_parser;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, stdin, Error, Write, BufRead, BufReader};
use std::process::exit;
use std::cmp::Ordering;
use arg_parser::ArgParser;
use std::fs::File;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = r#"
NAME
    sort - sort lines of text files

SYNOPSIS
    sort [ -h | --help | -n ] [FILE]...

DESCRIPTION
    Write sorted concatenation of FILE(s) to standard output.

    With no FILE, read standard input.

OPTIONS
    -h
    --help
        display this help and exit
    -n
    --numeric-sort
        sort numerically
"#; /* @MANEND */

fn get_first_f64(a: &str) -> f64 {
    for s in a.split_whitespace() {
        match s.parse::<f64>() {
            Ok(a) => return a,
            Err(_) => (),
        }
    }
    return std::f64::NEG_INFINITY;
}

fn numeric_compare(a: &String, b: &String) -> Ordering {
    let fa = get_first_f64(a);
    let fb = get_first_f64(b);

    if fa > fb {
        Ordering::Greater
    } else if fa < fb {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

fn lines_from_stdin() -> Result<Vec<String>, Error> {
    let stdin = stdin();
    let mut lines = Vec::new();

    let f = BufReader::new(stdin.lock());
    for line in f.lines() {
        match line {
            Ok(l) => lines.push(l),
            Err(e) => return Err(e),
        }
    }
    Ok(lines)
}

fn lines_from_files(paths: &Vec<&String>) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();

    for path in paths {
        let f = try!(File::open(path));
        let f = BufReader::new(f);
        for line in f.lines() {
            match line {
                Ok(l) => lines.push(l),
                Err(e) => return Err(e),
            }
        }
    }
    Ok(lines)
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(2)
        .add_flag(&["n", "numeric-sort"])
        .add_flag(&["u", "unique"])
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
            if parser.found("numeric-sort") {
                l.sort_by(numeric_compare);
            } else {
                l.sort();
            }
            if parser.found("unique") {
                l.dedup();
            }
            for x in l {
                let _ = writeln!(stdout, "{}", x);
            }
        }
        Err(e) => {
            let _ = write!(stderr, "{}", e);
        }
    }
}
