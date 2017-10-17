#![deny(warnings)]

extern crate extra;
extern crate arg_parser;
#[macro_use]
extern crate coreutils;

use std::io::Read;
use std::fs::File;
use std::string::String;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;

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
    let mut parser = ArgParser::new(7)
        .add_flag(&["a", "all"])
        .add_flag(&["m", "machine"])
        .add_flag(&["n", "nodename"])
        .add_flag(&["r", "kernel-release"])
        .add_flag(&["s", "kernel-name"])
        .add_flag(&["v", "kernel-version"])
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("uname"), MAN_PAGE);

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

    println!("{}", uname.join(" "));
}
