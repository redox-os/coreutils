#![deny(warnings)]
extern crate coreutils;
extern crate extra;

use std::env;
use std::fs::File;
use std::io::{stderr, stdout, Read, Write};
use std::time::Instant;
use std::process::exit;
use coreutils::{ArgParser, to_human_readable_string};
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = /* @MANSTART{du} */ r#"
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
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(5)
        .add_flag(&["h", "help"])
        .add_opt_default("", "bs", "512")
        .add_opt_default("", "count", "-1")
        .add_opt("", "if")
        .add_opt("", "of");
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if !parser.found("if") || !parser.found("of") {
        fail("missing if or of", &mut stderr);
    }

    let bs: usize = parser.get_opt("bs").unwrap().parse::<usize>().unwrap();
    let count = parser.get_opt("count").unwrap().parse::<i32>().unwrap();

    let in_path: String = parser.get_opt("if").unwrap();
    let out_path = parser.get_opt("of").unwrap();
    let status = 1;

    let mut input = File::open(in_path).expect("dd: failed to open if");
    let mut output = File::create(out_path).expect("dd: failed to open of");

    let mut in_recs = 0;
    let mut in_extra = 0;
    let mut out_recs = 0;
    let mut out_extra = 0;
    let mut out_total = 0;
    let mut running = true;
    let mut buffer = vec![0; bs];
    let mut last_print = 0;
    let mut last_print_out = 0;
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

        if status >= 2 {
            let elapsed = start.elapsed().as_secs();
            if elapsed > last_print {
                let _ = write!(stderr, "\r\x1B[K{} bytes ({}B) copied, {} s, {}B/s", out_total, to_human_readable_string(out_total), elapsed, to_human_readable_string(out_total - last_print_out));
                let _ = stderr.flush();
                last_print = elapsed;
                last_print_out = out_total;
            }
        }
    }

    if status >= 1 {
        let elapsed_duration = start.elapsed();
        let elapsed = elapsed_duration.as_secs() as f64 + (elapsed_duration.subsec_nanos() as f64)/1000000000.0;

        if status >= 2 && last_print > 0 {
            let _ = writeln!(stderr, "");
        }

        let _ = writeln!(stderr, "{}+{} records in", in_recs, in_extra);
        let _ = writeln!(stderr, "{}+{} records out", out_recs, out_extra);
        let _ = writeln!(stderr, "{} bytes ({}B) copied, {} s, {}B/s", out_total, to_human_readable_string(out_total), elapsed, to_human_readable_string((out_total as f64/elapsed) as u64));
    }
}
