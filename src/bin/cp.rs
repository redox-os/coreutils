extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom};
use coreutils::extra::{OptionalExt, fail};

fn main() {
    let ref src = env::args().nth(1).fail("no source argument.");
    let dsts = env::args().skip(2);

    if dsts.len() < 1 {
        fail("cp: no destination arguments");
    }

    let mut src_file = fs::File::open(src).try();
    for ref dst in dsts {
        let mut dst_file = fs::File::create(dst).try();

        src_file.seek(SeekFrom::Start(0)).try();
        io::copy(&mut src_file, &mut dst_file).try();
    }
}
