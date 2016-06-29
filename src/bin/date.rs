#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use extra::option::OptionalExt;
use std::process::exit;

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

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for arg in env::args().skip(1){
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        }
    }

    let time = SystemTime::now();
    let duration = time.duration_since(UNIX_EPOCH).try(&mut stderr);

    stdout.write(&format!("{}\n", duration.as_secs()).as_bytes()).try(&mut stderr);
}
