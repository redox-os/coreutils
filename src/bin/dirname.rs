#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;

const MAN_PAGE: &'static str = /* @MANSTART{dirname} */ r#"
NAME
    dirname - strip last component from file name

SYNOPSIS
    dirname [ -h | --help ] FILE...

DESCRIPTION
    Strip last component from file name.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("dirname"), MAN_PAGE);

    for path in &parser.args[0..] {
        if path.len() != 0 && path.chars().all(|c| c == '/') {
            println!("/");
        } else {
            let path = path.trim_right_matches('/');
            if let Some(end) = path.rfind('/') {
                println!("{}", &path[..end]);
            } else {
                println!(".");
            }
        }
    }
}
