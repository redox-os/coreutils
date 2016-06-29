#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::Command;
use std::time::Instant;
use extra::option::OptionalExt;
use std::process::exit;

const MAN_PAGE: &'static str = /* @MANSTART{time} */ r#"
NAME
    time - timer for commands

SYNOPSIS
    time [ -h | --help ][COMMAND] [ARGUEMENT]...

DESCRIPTION
    Runs the command taken as the first arguement and outputs the time the command took to execute.

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

    let time = Instant::now();

    let mut args = env::args().skip(1);
    if let Some(name) = args.next() {
        let mut command = Command::new(&name);
        for arg in args {
            command.arg(&arg);
        }
        command.spawn().try(&mut stderr).wait().try(&mut stderr);
    }

    let duration = time.elapsed();
    stdout.write(&format!("{}m{:.3}s\n", duration.as_secs() / 60,
                                   (duration.as_secs()%60) as f64 + (duration.subsec_nanos() as f64)/1000000000.0
                ).as_bytes()).try(&mut stderr);
}
