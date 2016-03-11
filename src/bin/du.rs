#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use coreutils::extra::{OptionalExt, WriteExt};

fn print_path(path: &str) {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut entries = Vec::new();

    let dir = fs::read_dir(path).try(&mut stderr);

    for entry_result in dir {
        let entry = entry_result.try(&mut stderr);

        let file_type = entry.file_type().try(&mut stderr);
        let directory = file_type.is_dir();

        if let Some(path_str) = entry.file_name().to_str() {
            entries.push(path_str.to_owned());
            if directory {
                entries.last_mut().unwrap().push('/');
            }
        } else {
            stderr.writeln(b"warning: failed to convert path to a valid string.").try(&mut stderr);
        }
    }

    entries.sort();

    for entry in entries.iter() {
        let mut entry_path = path.to_owned();
        if !entry_path.ends_with('/') {
            entry_path.push('/');
        }
        entry_path.push_str(entry);

        let metadata = fs::metadata(&entry_path).try(&mut stderr);
        let size = metadata.len();

        stdout.write(((size + 1023) / 1024).to_string().as_bytes()).try(&mut stderr);
        stdout.write(b"\t").try(&mut stderr);
        stdout.writeln(entry.as_bytes()).try(&mut stderr);
    }
}
fn main() {
    if let Some(ref x) = env::args().nth(1) {
        print_path(x);
    } else {
        print_path(".");
    }
}
