#![deny(warnings)]

extern crate arg_parser;
#[macro_use]
extern crate coreutils;
extern crate extra;

use std::io::{stdout, stderr, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use coreutils::format_time;
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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("date"), MAN_PAGE);

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

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
