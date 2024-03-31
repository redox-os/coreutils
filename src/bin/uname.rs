extern crate extra;
extern crate arg_parser;

use std::io::{self, Write};
use std::env;
use std::process;
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
    let mut parser = ArgParser::new(7)
        .add_flag(&["a", "all"])
        .add_flag(&["m", "machine"])
        .add_flag(&["n", "nodename"])
        .add_flag(&["r", "kernel-release"])
        .add_flag(&["s", "kernel-name"])
        .add_flag(&["v", "kernel-version"])
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        process::exit(0);
    }

    if parser.found("all") {
        *parser.flag("machine") = true;
        *parser.flag("nodename") = true;
        *parser.flag("kernel-release") = true;
        *parser.flag("kernel-name") = true;
        *parser.flag("kernel-version") = true;
    } else if !(parser.found("machine") || parser.found("nodename")
                || parser.found("kernel-release") || parser.found("kernel-name")
                || parser.found("kernel-version")) {
        *parser.flag("kernel-name") = true;
    }

    let mut file = File::open("sys:uname").unwrap();
    let mut uname_str = String::new();
    file.read_to_string(&mut uname_str).unwrap();
    let mut lines = uname_str.lines();

    let mut uname = Vec::new();

    let kernel = lines.next().unwrap();
    if parser.found("kernel-name") {
        uname.push(kernel);
    }
    let nodename = lines.next().unwrap();
    if parser.found("nodename") {
        uname.push(nodename);
    }
    let release = lines.next().unwrap();
    if parser.found("kernel-release") {
        uname.push(release);
    }
    let version = lines.next().unwrap();
    if parser.found("kernel-version") {
        uname.push(version);
    }
    let machine = lines.next().unwrap();
    if parser.found("machine") {
        uname.push(machine);
    }

    let _ = writeln!(stdout, "{}", uname.join(" "));
}
