#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::env;
use std::path::Path;
use std::io::stderr;
use std::fs::{remove_dir_all, hard_link};
use std::os::unix::fs::symlink;

use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = r#"
NAME
    ln - make links between files

SYNOPSIS
    ln [OPTIONS] TARGET LINK_NAME
    ln [OPTIONS] TARGET

DESCRIPTION
    Create links between files.

OPTIONS
    -f --force
        Force removing existing destination fila
    -s --symbolic
        Make symbolic link instead of hard link
    --help
        show this help and exit
"#; /* @MANEND */


fn main() {
    let mut parser = ArgParser::new(6)
        .add_flag(&["f", "force"])
        .add_flag(&["s", "symbolic"])
        .add_flag(&["", "help"]);
    parser.process_common(help_info!("ln"), MAN_PAGE);

    let mut stderr = stderr();

    let mut cwd = env::current_dir().expect("can't get cwd");
    let src = Path::new(&parser.args[0]);
    let dst = match parser.args.len() {
        2 => Path::new(&parser.args[1]),
        1 => {
            cwd.push(src.file_name().try(&mut stderr));
            cwd.as_path()
        }
        _ => fail("use --help", &mut stderr),
    };

    let mut dst = dst.to_owned();
    if dst.is_dir() {
        dst.push(src.file_name().unwrap_or(src.as_os_str()));
    }

    if parser.found("force") {
        remove_dir_all(&dst).try(&mut stderr);
    }

    if parser.found("symbolic") {
        symlink(src, &dst).try(&mut stderr);
    } else {
        hard_link(src, &dst).try(&mut stderr);
    }

}
