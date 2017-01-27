#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{date} */ r#"
NAME
    date - prints the system time

SYNOPSIS
    date [ -h | --help]

DESCRIPTION
    Prints the system time and date

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

// Sweet algorithm from http://ptspts.blogspot.com/2009/11/how-to-convert-unix-timestamp-to-civil.html
// TODO: Apply timezone offset
fn format_time(mut ts: i64) -> String {
    let s = ts%86400;
    ts /= 86400;
    let h = s/3600;
    let m = s/60%60;
    let s = s%60;
    let x = (ts*4+102032)/146097+15;
    let b = ts+2442113+x-(x/4);
    let mut c = (b*20-2442)/7305;
    let d = b-365*c-c/4;
    let mut e = d*1000/30601;
    let f = d-e*30-e*601/1000;
    if e < 14 {
        c -= 4716;
        e -= 1;
    } else {
        c -= 4715;
        e -= 13;
    }
    format!("{:>04}-{:>02}-{:>02} {:>02}:{:>02}:{:>02}", c, e, f, h, m, s)
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found(&'h') || parser.found("help") {
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
    let mut ts = duration.as_secs() as i64;
    ts += tz_offset * 3600;

    stdout.write(format!("{} {}\n", format_time(ts), tz_name).as_bytes()).try(&mut stderr);
}
