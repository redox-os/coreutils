#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom, stdout};
use coreutils::extra::{OptionalExt, fail};

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let ref src = env::args().nth(1).fail("no source argument.", &mut stdout);
    let dsts = env::args().skip(2);

    if dsts.len() < 1 {
        fail("no destination arguments.", &mut stdout);
    }

    let mut src_file = fs::File::open(src).try(&mut stdout);
    for ref dst in dsts {
        let mut dst_file = fs::File::create(dst).try(&mut stdout);

        src_file.seek(SeekFrom::Start(0)).try(&mut stdout);
        io::copy(&mut src_file, &mut dst_file).try(&mut stdout);
    }
}
