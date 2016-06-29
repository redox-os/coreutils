#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom, stderr, stdout, Write};
use extra::io::fail;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{cp} */ r#"
NAME
    cp - copy files

SYNOPSIS
    cp SOURCE_FILE DESTINATION_FILES...

DESCRIPTION
    The cp utility copies the contents of the SOURCE_FILE to the DESTINATION_FILES. If multiple
    destionation files are specified, then multiple copies of SOURCE_FILE are created.

OPTIONS
    -h
    --help
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
    let dsts = env::args().skip(2);

    if dsts.len() < 1 {
        fail("No destination arguments. Use --help to see the usage.", &mut stderr);
    }

    let mut src_file = fs::File::open(src).try(&mut stderr);
    for ref dst in dsts {
        let mut dst_file = fs::File::create(dst).try(&mut stderr);

        src_file.seek(SeekFrom::Start(0)).try(&mut stderr);
        io::copy(&mut src_file, &mut dst_file).try(&mut stderr);
    }
}
