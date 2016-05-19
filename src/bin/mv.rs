#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stderr, stdout, Write};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{mv} */ r#"
NAME
    mv - move files

SYNOPSIS
    mv [ -h | --help ] SOURCE_FILE DESTINATION_FILE

DESCRIPTION
    The mv utility renames the file named by the SOURCE_FILE operand to the destination path named by the DESTINATION_FILE operand.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    if env::args().count() == 2 {
        if let Some(arg) = env::args().nth(1) {
            if arg == "--help" || arg == "-h" {
                stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                return;
            }
        }
    }

    let ref src = env::args().nth(1).fail("No source argument. Use --help to see the usage.", &mut stderr);
    let ref dst = env::args().nth(2).fail("No destination argument. Use --help to see the usage.", &mut stderr);

    fs::rename(src, dst).try(&mut stderr);
}
