#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::io::{stdout, stderr, Write};
use std::process::Command;
use std::time::Instant;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;

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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("time"), MAN_PAGE);

    let time = Instant::now();

    if !parser.args.is_empty() {
        let mut command = Command::new(&parser.args[0]);
        for arg in &parser.args[1..] {
            command.arg(arg);
        }
        command.spawn().try(&mut stderr).wait().try(&mut stderr);
    }

    let duration = time.elapsed();
    stdout.write(&format!("{}m{:.3}s\n", duration.as_secs() / 60,
                          (duration.as_secs()%60) as f64 + (duration.subsec_nanos() as f64)/1000000000.0)
        .as_bytes()).try(&mut stderr);
}
