#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use coreutils::{ArgParser, format_time};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{date} */ r#"
NAME
    date - prints the system time

SYNOPSIS
    date [ -h | --help] [offset]

DESCRIPTION
    Prints the system time and date

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let mut tz_offset = 0;
    for arg in &parser.args {
        if let Ok(offset) = arg.parse::<i64>() {
            tz_offset = offset;
        }
    }

    let tz_name = if tz_offset == 0 {
        format!("UTC")
    } else if tz_offset > 0 {
        format!("UTC+{:>02}", tz_offset)
    } else {
        format!("UTC-{:>02}", -tz_offset)
    };

    let time = SystemTime::now();
    let duration = time.duration_since(UNIX_EPOCH).try(&mut stderr);
    let ts = duration.as_secs() as i64;

    stdout.write(format!("{} {}\n", format_time(ts, tz_offset), tz_name).as_bytes()).try(&mut stderr);
}
