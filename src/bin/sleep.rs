#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;


use coreutils::extra::{OptionalExt};

static MAN_PAGE: &'static str = r#"NAME
    sleep - delay for a specified amount of time

SYNOPSIS
    sleep NUMBER[SUFFIX]...
    sleep OPTION

DESCRIPTION
    Pause the shell for NUMBER seconds. An optional SUFFIX may be applied, such as 's' for seconds (default), 'm' for minutes, 'h' for hours or 'd' for day. Given multiple arguments, it will pause for the amount of time specified by the sum of their values. This implementation supports floating point numbers as input.

    Example: sleep 60; sleep 1m; sleep 30s 30s;
             sleep 0.5; sleep 0.5s; sleep 0.25s 0.25s

OPTIONS
    -h, --help
        display this help and exit

AUTHOR
    Written by Jeremy Soller and Michael Murphy.
"#;

static MISSING_OPERAND: &'static str       = "sleep: missing operand";
static HELP_INFO: &'static str             = "Try 'sleep --help' for more information.";

fn main() {
    let stdout     = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut args   = env::args().skip(1);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            },
            _ => {
                // TODO: (not supported in Redox due to missing _mulodi4) thread::sleep(Duration::new(arg.parse().try(), 0))
                thread::sleep(Duration::from_millis(argument_to_ms(&arg)));
                for argument in args {
                    thread::sleep(Duration::from_millis(argument_to_ms(&argument)));
                }
            }
        }
    } else {
        stdout.write(MISSING_OPERAND.as_bytes()).try(&mut stderr);
        stdout.write(HELP_INFO.as_bytes()).try(&mut stderr);
    }
}

/// Check if the argument uses a time suffix and convert the time accordingly.
fn argument_to_ms(argument: &str) -> u64 {
    // If the argument is a number, the duration is in seconds, so multiply it by 1000.
    if let Ok(number) = argument.parse::<f64>() {
        return (number * 1000f64) as u64;
    }

    // Split the argument into two strings at the last character. The first string should be the
    // number while the second string should be the duration unit:
    // s = seconds; m = minutes; h = hours; d = days
    let (prefix, suffix) = argument.split_at(argument.len()-1);
    if let Ok(number) = prefix.parse::<f64>() {
        match suffix {
            "s" => (number * 1000f64) as u64,
            "m" => (number * 60000f64) as u64,
            "h" => (number * 3600000f64) as u64,
            "d" => (number * 86400000f64) as u64,
            _   => {
                println!("sleep: invalid time interval '{}'", argument);
                std::process::exit(1);
            }
        }
    } else {
        println!("sleep: invalid time interval '{}'", argument);
        std::process::exit(1);
    }
}
