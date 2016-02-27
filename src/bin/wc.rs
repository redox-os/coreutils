#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::{self, stdout, stderr, Write};
use std::iter;
use std::process::exit;

use coreutils::extra::{OptionalExt, print, println};

static MAN_PAGE: &'static str = r#"
    NAME
        wc - count words, bytes and lines of a file or byte stream.
    SYNOPSIS
        wc [-h | --help] [-c | --bytes] [-w | --words] [-l | --lines] [FILE 1] [FILE 2]...
    DESCRIPTION
        This utility will dump word count, line count, and byte count of a file or byte stream. If multiple files are given, it will print a total count. If no file name is given, 'wc' reads from stdin until EOF.

        Flags can be arbitrarily combined into a single flag, for example, '-c -w' (count bytes and count words) can be reduced to '-cw'.

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
"#;

#[derive(Copy, Clone)]
struct Flags {
    count_lines: bool,
    count_words: bool,
    count_bytes: bool,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            count_lines: false,
            count_words: false,
            count_bytes: false,
        }
    }

    fn print_count<'a, W: Write>(self, lines: u64, words: u64, bytes: u64, file: &'a str, stdout: &mut W) {
        print(b"    ", &mut *stdout);

        if self.count_lines {
            print(lines.to_string().as_bytes(), &mut *stdout);
            print(b" ", &mut *stdout);
        }
        if self.count_words {
            print(words.to_string().as_bytes(), &mut *stdout);
            print(b" ", &mut *stdout);
        }
        if self.count_bytes {
            print(bytes.to_string().as_bytes(), &mut *stdout);
            print(b" ", &mut *stdout);
        }

        println(file.as_bytes(), &mut *stdout);
    }

    fn default_to(&mut self) {
        // Defaults to behavior of -lwc
        if !(self.count_lines || self.count_words || self.count_bytes) {
            self.count_lines = true;
            self.count_words = true;
            self.count_bytes = true;
        }
    }
}

fn main() {
    let mut opts = Flags::new();
    let mut first_file  = String::new();
    let mut args = env::args().skip(1);
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stderr = stderr();
    let mut stderr = stderr.lock();

    loop { // To avoid consumption of the iter, we use loop.
        let arg = if let Some(x) = args.next() { x } else { break };

        // TODO add -m, maybe add -L

        if arg.starts_with("-") {
            match arg.as_str() {
                "--lines" | "-l" => opts.count_lines = true,
                "--words" | "-w" => opts.count_words = true,
                "--bytes" | "-c" => opts.count_bytes = true,
                "--help"  | "-h" => {
                    print(MAN_PAGE.as_bytes(), &mut stdout);
                    return;
                },
                _ => {
                    print(b"error: unknown flag, ", &mut stderr);
                    println(arg.as_bytes(), &mut stderr);
                    stderr.flush().try(&mut stderr);
                    exit(1);
                },
            }
        } else { // This is a file name
            first_file = arg;
            break;
        }
    }

    opts.default_to();

    if first_file == "" {
        let stdin = io::stdin();
        let stdin = stdin.lock();

        let (lines, words, bytes) = do_count(stdin);
        opts.print_count(lines, words, bytes, "stdin", &mut stdout);
    } else {
        let mut total_lines = 0;
        let mut total_words = 0;
        let mut total_bytes = 0;

        let single_file = args.len() == 1;

        for path in iter::once(first_file).chain(args) {
            //TODO would be easy to use stdin for - but
            //that is probably something the shell should handle?
            //unix it's all just fds so it's whatever dunno here tho
            //(also - is specific to sh/bash fwiw).

            let file = File::open(&path).try(&mut stderr);
            let (lines, words, bytes) = do_count(file);

            total_lines += lines;
            total_words += words;
            total_bytes += bytes;

            opts.print_count(lines, words, bytes, &path, &mut stdout);
        }

        if single_file {
            opts.print_count(total_lines, total_words, total_bytes, "total", &mut stdout);
        }
    }
}

fn is_whitespace(byte: u8) -> bool {
    //FIXME this works like iswspace w/ default C locale
    //but not good enough for en_US.UTF8 among others

    byte == b'\n' // newline
    || byte == b'\t' // tab
    || byte == b'\r' // cr
    || byte == 0xc // formfeed
    || byte == 0xb // vtab
    || byte == b' ' // space
}

fn do_count<T: std::io::Read>(input: T) -> (u64, u64, u64) {
    let mut line_count = 0;
    let mut word_count = 0;
    let mut byte_count = 0;
    let mut got_space = true;

    for byte_result in input.bytes() {
        if let Ok(byte) = byte_result {
            if byte == b'\n' {
                line_count += 1;
            }

            if is_whitespace(byte) {
                got_space = true;
            } else if got_space {
                got_space = false;
                word_count += 1;
            }

            byte_count += 1;
        }
    }

    (line_count, word_count, byte_count)
}
