#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::io::{self, stdout, stderr, Read, Write, Stderr};
use std::ops::{Add, AddAssign};
use std::process::exit;
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::WriteExt;

static MAN_PAGE: &'static str = /* @MANSTART{wc} */ r#"
NAME
    wc - count words, bytes and lines of a file or byte stream.

SYNOPSIS
    wc [-h | --help] [-c | --bytes] [-w | --words] [-l | --lines] [FILE 1] [FILE 2]...

DESCRIPTION
    This utility will dump word count, line count, and byte count of a file or byte stream. If
    multiple files are given, it will print a total count. If no file name is given, 'wc' reads
    from stdin until EOF.

    Flags can be arbitrarily combined into a single flag, for example, '-c -w' (count bytes and
    count words) can be reduced to '-cw'.

    If no flags are given, 'wc' defaults to '-cwl'.

OPTIONS
    -h
    --help
        Print this manual page.

    -c
    --bytes
        Count bytes.

    -w
    --words
        Count words (i.e. contiguous strings seperated by whitespaces).

    -l
    --lines
        Count lines (seperated by NL).

AUTHOR
    This program was written mainly by Alice Maz.
"#; /* @MANEND */

#[derive(Copy, Clone, Debug, Default)]
struct Counter {
    lines: u64,
    words: u64,
    bytes: u64,
}

impl Counter {
    #[inline]
    fn new<T: Read>(input: T) -> Self {
        let mut line_count = 0;
        let mut word_count = 0;
        let mut byte_count = 0;
        let mut got_space = true;
        let mut input_bytes = input.bytes();

        while let Some(Ok(byte)) = input_bytes.next() {
            if byte == b'\n' {
                line_count += 1;
            }

            if byte.is_whitespace() {
                got_space = true;
            } else if got_space {
                got_space = false;
                word_count += 1;
            }

            byte_count += 1;
        }

        Counter { lines: line_count, words: word_count, bytes: byte_count }
    }
}

impl Add for Counter {
    type Output = Counter;

    fn add(self, other: Self) -> Self {
        Counter {
            lines: self.lines + other.lines,
            words: self.words + other.words,
            bytes: self.bytes + other.bytes
        }
    }
}

impl AddAssign for Counter {
    fn add_assign(&mut self, _rhs: Self) {
        self.lines += _rhs.lines;
        self.words += _rhs.words;
        self.bytes += _rhs.bytes;
    }
}

fn u64_num_digits(val: u64) -> usize {
    if val < 10 {
        1
    } else {
        1 + u64_num_digits(val / 10)
    }
}

fn print_counts<W: Write>(parser: &ArgParser, mut counts: Vec<(Counter, String)>, stdout: &mut W, stderr: &mut Stderr) {
    use std::cmp::max;

    let mut max_lines_digits = 0;
    let mut max_words_digits = 0;
    let mut max_bytes_digits = 0;

    for &mut (count, _) in &mut counts {
        if parser.found(&'l') || parser.found("lines") {
            max_lines_digits = max(max_lines_digits, u64_num_digits(count.lines));
        }
        if parser.found(&'w') || parser.found("words") {
            max_words_digits = max(max_words_digits, u64_num_digits(count.words));
        }
        if parser.found(&'c') || parser.found("bytes") {
            max_bytes_digits = max(max_bytes_digits, u64_num_digits(count.bytes));
        }
    }

    for &mut (count, ref mut path) in &mut counts {
        print_count(&parser, count, path, Counter {
            lines:
                if parser.found(&'l') || parser.found("lines") {
                    (max_lines_digits - u64_num_digits(count.lines) + 1) as u64
                } else {
                    0
                },
            words:
                if parser.found(&'w') || parser.found("words") {
                    (max_words_digits - u64_num_digits(count.words) + 1) as u64
                } else {
                    0
                },
            bytes:
                if parser.found(&'c') || parser.found("bytes") {
                    (max_bytes_digits - u64_num_digits(count.bytes) + 1) as u64
                } else {
                    0
                },
        },
        stdout, stderr)
    }
}

fn print_count<'a, W: Write>(parser: &ArgParser, count: Counter, path: &'a str, padding: Counter, stdout: &mut W, stderr: &mut Stderr) {
    stdout.write(b"    ").try(stderr);

    if parser.found(&'l') || parser.found("lines") {
        stdout.write(count.lines.to_string().as_bytes()).try(stderr);
        for _ in 0..padding.lines {
            stdout.write(b" ").try(stderr);
        }
    }
    if parser.found(&'w') || parser.found("words") {
        stdout.write(count.words.to_string().as_bytes()).try(stderr);
        for _ in 0..padding.words {
            stdout.write(b" ").try(stderr);
        }
    }
    if parser.found(&'c') || parser.found("bytes") {
        stdout.write(count.bytes.to_string().as_bytes()).try(stderr);
        for _ in 0..padding.bytes {
            stdout.write(b" ").try(stderr);
        }
    }

    stdout.writeln(path.as_bytes()).try(&mut *stderr);
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(4)
        .add_flag("l", "lines")
        .add_flag("w", "words")
        .add_flag("c", "bytes")
        .add_flag("h", "help");
    parser.parse(env::args());

    if parser.found(&'h') || parser.found("help") {
        stdout.writeln(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if !(parser.found(&'l') || parser.found("lines") ||
         parser.found(&'w') || parser.found("words") ||
         parser.found(&'c') || parser.found("bytes")) {
        *parser.flag(&'l') = true;
        *parser.flag(&'w') = true;
        *parser.flag(&'c') = true;
        *parser.flag("lines") = true;
        *parser.flag("words") = true;
        *parser.flag("bytes") = true;
    }

    if parser.args.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();

        print_count(&parser,
            Counter::new(stdin),
            "stdin",
            Counter {lines: 1, words: 1, bytes: 1},
            &mut stdout,
            &mut stderr);
    } else {
        let mut files = Vec::new();
        let mut total_count = Counter::default();

        for path in parser.args.iter() {
            //TODO would be easy to use stdin for - but
            //that is probably something the shell should handle?
            //unix it's all just fds so it's whatever dunno here tho
            //(also - is specific to sh/bash fwiw).

            let file = File::open(path).try(&mut stderr);
            let file_count = Counter::new(file);
            total_count += file_count;

            files.push((file_count, path.to_owned()));
        }

        if parser.args.len() > 1 {
            files.push((total_count, "total".to_owned()));
        }

        print_counts(&parser, files, &mut stdout, &mut stderr);
    }
}

pub trait AsciiWhitespace { fn is_whitespace(self) -> bool; }

impl AsciiWhitespace for u8 {
    fn is_whitespace(self) -> bool {
        //FIXME this works like iswspace w/ default C locale
        //but not good enough for en_US.UTF8 among others

        self == b'\n' // newline
        || self == b'\t' // tab
        || self == b'\r' // cr
        || self == 0xc // formfeed
        || self == 0xb // vtab
        || self == b' ' // space
    }
}
