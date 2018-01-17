#![deny(warnings)]

extern crate extra;
extern crate arg_parser;

use std::io::{self, Write};
use std::env::args;
use std::fs::File;
use std::string::String;
use arg_parser::ArgParser;
use extra::option::OptionalExt;
use std::io::Read;

const MAN_PAGE: &'static str = /* @MANSTART{uname} */ r#"
NAME
    uname - print system information

SYNOPSIS
    uname [-a] [-m] [-n] [-r] [-s] [-v]

DESCRIPTION
    Print system information.

OPTIONS
    -a
        print all

    -m
        machine

    -n
        nodename

    -r
        release

    -s
        kernel name

    -v
        version

    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let potential_arguments = ["all", "machine", "nodename", "kernel-release", 
                     "kernel-name", "kernel-version"];
    let mut parser = ArgParser::new(7)
        .add_flag(&["a", "all"])
        .add_flag(&["m", "machine"])
        .add_flag(&["n", "nodename"])
        .add_flag(&["r", "kernel-release"])
        .add_flag(&["s", "kernel-name"])
        .add_flag(&["v", "kernel-version"])
        .add_flag(&["h", "help"]);
    parser.parse(args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
    } else {
        if parser.found("all") {
            for i in 1..6 {
                *parser.flag(potential_arguments[i]) = true; //Machine, nodename, kernel-release, kernel-name, kernel-version.
            }
        } else if args().len() == 1 {
            *parser.flag("kernel-name") = true;
        }

        let mut file = File::open("sys:uname").unwrap();
        let mut uname_str = String::new();
        file.read_to_string(&mut uname_str).unwrap();
        let mut lines = uname_str.lines();

        let mut uname = Vec::new();

        for i in 1..potential_arguments.len() { 
            let uname_segment = lines.next().unwrap();
            if parser.found(potential_arguments[i]) {
                uname.push(uname_segment);
            }
        }

        println!("{}", uname.join(" "));
    }
}
