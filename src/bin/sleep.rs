#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::io::{self, Write, Stderr};
use std::process::exit;
use std::thread;
use std::time::Duration;

use coreutils::extra::OptionalExt;

const MAN_PAGE: &'static str = r#"NAME
    sleep - delay for a specified amount of time.

SYNOPSIS
    sleep [-h | --help] NUMBER[SUFFIX]...

DESCRIPTION
    Pause the shell for NUMBER seconds. An optional SUFFIX may be applied, such as 's' for seconds (default), 'm' for minutes, 'h' for hours or 'd' for day. Given multiple arguments, it will pause for the amount of time specified by the sum of their values. This implementation supports floating point numbers as input.

EXAMPLE
    The following are three possible arguments with the same effect:

    sleep {90, 1.5m, 1m 30s}

OPTIONS
    -h
    --help
        display this help and exit

AUTHOR
    Written by Michael Murphy.
"#;

const MISSING_OPERAND: &'static str = "missing operand\n";
const HELP_INFO:       &'static str = "Try 'sleep --help' for more information.\n";

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
                thread::sleep(Duration::from_millis(argument_to_ms(&arg, &mut stderr)));
                for argument in args {
                    thread::sleep(Duration::from_millis(argument_to_ms(&argument, &mut stderr)));
                }
            }
        }
    } else {
        stderr.write(MISSING_OPERAND.as_bytes()).try(&mut stderr);
        stderr.write(HELP_INFO.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        exit(1);
    }
}

/// Check if the argument uses a time suffix and convert the time accordingly.
fn argument_to_ms(argument: &str, stderr: &mut Stderr) -> u64 {
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
                stderr.write(b"invalid time interval '").try(&mut *stderr);
                stderr.write(argument.as_bytes()).try(&mut *stderr);
                stderr.write(b"\'\n").try(&mut *stderr);
                stderr.flush().try(&mut *stderr);
                exit(1);
            }
        }
    } else {
        stderr.write(b"invalid time interval '").try(&mut *stderr);
        stderr.write(argument.as_bytes()).try(&mut *stderr);
        stderr.write(b"\'\n").try(&mut *stderr);
        stderr.flush().try(&mut *stderr);
        exit(1);
    }
}
