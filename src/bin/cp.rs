#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom, stderr};
use coreutils::extra::{OptionalExt, fail};

fn main() {
    let stderr = stderr();
    let mut stderr = stderr.lock();
    let ref src = env::args().nth(1).fail("no source argument.", &mut stderr);
    let dsts = env::args().skip(2);

    if dsts.len() < 1 {
        fail("no destination arguments.", &mut stderr);
    }

    let mut src_file = fs::File::open(src).try(&mut stderr);
    for ref dst in dsts {
        let mut dst_file = fs::File::create(dst).try(&mut stderr);

        src_file.seek(SeekFrom::Start(0)).try(&mut stderr);
        io::copy(&mut src_file, &mut dst_file).try(&mut stderr);
    }
}
