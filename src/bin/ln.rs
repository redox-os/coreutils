#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;

use std::env;
use std::path::Path;
use std::io::{stdout, stderr, Write};
use std::fs::{remove_dir_all, hard_link};
use std::os::unix::fs::symlink;
use std::process::exit;

use arg_parser::ArgParser;
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
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(6)
        .add_flag(&["f", "force"])
        .add_flag(&["s", "symbolic"])
        .add_flag(&["", "help"]);
    parser.parse(env::args());

    if parser.found("help") || parser.args.len() == 0 {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    } else {

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

        if parser.found("force") {
            remove_dir_all(dst).try(&mut stderr);
        }
        if parser.found("symbolic") {
            symlink(src, dst).try(&mut stderr);
        } else {
            hard_link(src, dst).try(&mut stderr);
        }
    }

}
