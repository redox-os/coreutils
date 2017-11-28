#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stderr, stdin, stdout, Read, Write};
use std::time::Instant;
use std::process::exit;
use arg_parser::ArgParser;
use coreutils::to_human_readable_string;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{dd} */ r#"
NAME
    dd - copy a file

SYNOPSIS
    dd [ -h | --help ] if=[FILE] of=[FILE]

DESCRIPTION
    The dd tool copies from a file to another file in 512-byte block sizes

OPTIONS
    -h
    --help
        display this help and exit

    bs=n
        set input and output block size to n bytes
    count=n
        copy only n blocks
    if=file
        read from file instead of standard input
    of=file
        write output to file instead of standard out

"#; /* @MANEND */

fn main() {
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(5)
        .add_flag(&["h", "help"])
        .add_setting_default("bs", "512")
        .add_setting_default("count", "-1")
        .add_setting("if")
        .add_setting("of");
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let bs: usize = parser.get_setting("bs").unwrap().parse::<usize>().unwrap();
    let count = parser.get_setting("count").unwrap().parse::<i32>().unwrap();

    let mut input: Box<Read> = match parser.found("if") {
        true => {
            let path = parser.get_setting("if").unwrap();
            Box::new(File::open(path).expect("dd: failed to open if"))
        },
        false => Box::new(stdin),
    };

    let mut output: Box<Write> = match parser.found("of") {
        true => {
            let path = parser.get_setting("of").unwrap();
            Box::new(File::create(path).expect("dd: failed to open of"))
        },
        false => Box::new(stdout),
    };

    let mut in_recs = 0;
    let mut in_extra = 0;
    let mut out_recs = 0;
    let mut out_extra = 0;
    let mut out_total = 0;
    let mut running = true;
    let mut buffer = vec![0; bs];
    let start = Instant::now();
    while running {
        let in_count = input.read(&mut buffer).expect("dd: failed to read if");
        if in_count < bs {
            if in_count > 0 {
                in_extra += 1;
            }
            running = false;
        } else {
            in_recs += 1;
            if count != -1 {
                if in_recs >= count {
                    running = false;
                }
            }
        }

        let out_count = output.write(&buffer[.. in_count]).expect("dd: failed to write of");
        if out_count < bs {
            if out_count > 0 {
                out_extra += 1;
            }
            running = false;
        } else {
            out_recs += 1;
        }

        out_total += out_count as u64;
    }

    let elapsed_duration = start.elapsed();
    let elapsed = elapsed_duration.as_secs() as f64 + (elapsed_duration.subsec_nanos() as f64)/1000000000.0;
    let _ = writeln!(stderr, "{}+{} records in", in_recs, in_extra);
    let _ = writeln!(stderr, "{}+{} records out", out_recs, out_extra);
    let _ = writeln!(stderr, "{} bytes ({}B) copied, {} s, {}B/s", out_total, to_human_readable_string(out_total), elapsed, to_human_readable_string((out_total as f64/elapsed) as u64));
}
