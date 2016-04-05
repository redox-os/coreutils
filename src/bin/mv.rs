#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom, stderr};
use std::os::unix::fs::MetadataExt;
use extra::io::fail;
use extra::option::OptionalExt;


fn main() {
    let mut stderr = stderr();
    let mut arguments = env::args().skip(1).collect::<Vec<String>>();
    match arguments.len() {
        1 => fail("no source argument", &mut stderr),
        2 => fail("no destination argument", &mut stderr),
        _ => ()
    }

    let destination = arguments.pop().unwrap();
    for source in arguments {
        mv(&source, &destination, &mut stderr);
    }
}

/// If the source and destination are on the same device, rename the source file to prevent the
/// need to copy. Otherwise, if they are on different devices, copy the source to the destination.
/// In addition, if the destinaton is a directory, append the basename of the source path to the
/// destination path.
fn mv(src: &str, dst: &str, stderr: &mut io::Stderr) {
    let src_metadata = fs::metadata(src).fail("source doesn't exst", stderr);
    let dst_metadata = match fs::metadata(dst) {
        Ok(metadata) => metadata,
        Err(_)       => {
            let dst_file = fs::File::create(dst).try(stderr);
            dst_file.metadata().try(stderr)
        }
    };

    if src_metadata.dev() == dst_metadata.dev() {
        if dst_metadata.is_dir() {
            let src_base = match src.split('/').last() {
                Some(filename) => filename,
                None           => src
            };
            fs::rename(src, [dst, src_base].join("/")).try(stderr);
        } else {
            fs::rename(src, dst).try(stderr);
        }
    } else {
        let mut src_file = fs::File::open(src).try(stderr);
        let mut dst_file = if dst_metadata.is_dir() {
            let src_base = match src.split('/').last() {
                Some(filename) => filename,
                None           => src
            };
            fs::File::create([dst, src_base].join("/")).try(stderr)
        } else {
            fs::File::create(dst).try(stderr)
        };
        src_file.seek(SeekFrom::Start(0)).try(stderr);
        io::copy(&mut src_file, &mut dst_file).try(stderr);
    }
}
