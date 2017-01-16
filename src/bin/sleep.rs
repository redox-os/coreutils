//#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::io::{self, Write, Stderr};
use std::process::exit;
use std::thread;
use std::time::Duration;
use coreutils::ArgParser;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{sleep} */ r#"
NAME
    sleep - delay for a specified amount of time.

SYNOPSIS
    sleep [-h | --help] NUMBER[SUFFIX]...

DESCRIPTION
    Pause the shell for NUMBER seconds. An optional SUFFIX may be applied, such as 's' for seconds
    (default), 'm' for minutes, 'h' for hours or 'd' for day. Given multiple arguments, it will
    pause for the amount of time specified by the sum of their values. This implementation supports
    floating point numbers as input.

EXAMPLE
    The following are three possible arguments with the same effect:

    sleep {90, 1.5m, 1m 30s}

OPTIONS
    -h
    --help
        display this help and exit

AUTHOR
    Written by Michael Murphy.
"#; /* @MANEND */

const MISSING_OPERAND: &'static str = "missing operand\n";
const HELP_INFO:       &'static str = "Try 'sleep --help' for more information.\n";

fn main() {
    let stdout     = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.parse(env::args());

    if parser.found(&'h') || parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if !parser.args.is_empty() {
        let sleep_times_in_ms = parser.args.iter()
            .map(|argument| argument_to_ms(&argument, &mut stderr))
            .collect::<Vec<_>>();

        for sleep_time in sleep_times_in_ms {
            thread::sleep(Duration::from_millis(sleep_time));
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
        // Time to sleep must be positive, as we can't time travel yet
        if number.is_sign_positive() {
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
            stderr.write(b"negative ('").try(&mut *stderr);
            stderr.write(argument.as_bytes()).try(&mut *stderr);
            stderr.write(b"\') time intervals unsupported\n").try(&mut *stderr);
            stderr.flush().try(&mut *stderr);
            exit(1);
        }
    } else {
        stderr.write(b"invalid time interval '").try(&mut *stderr);
        stderr.write(argument.as_bytes()).try(&mut *stderr);
        stderr.write(b"\'\n").try(&mut *stderr);
        stderr.flush().try(&mut *stderr);
        exit(1);
    }
}
