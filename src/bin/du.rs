#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{stdout, Seek, SeekFrom};
use coreutils::extra::{OptionalExt, print, println};

fn print_path(path: &str) {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut entries = Vec::new();

    let dir = fs::read_dir(path).try(&mut stdout);

    for entry_result in dir {
        let entry = entry_result.try(&mut stdout);

        let file_type = entry.file_type().try(&mut stdout);
        let directory = file_type.is_dir();

        if let Some(path_str) = entry.file_name().to_str() {
            entries.push(path_str.to_owned());
            if directory {
                entries.last_mut().unwrap().push('/');
            }
        } else {
            println(b"warning: failed to convert path to a valid string.", &mut stdout);
        }
    }

    entries.sort();

    for entry in entries.iter() {
        let mut entry_path = path.to_string();
        if !entry_path.ends_with('/') {
            entry_path.push('/');
        }
        entry_path.push_str(entry);

        let mut file = fs::File::open(&entry_path).try(&mut stdout);
        let size = file.seek(SeekFrom::End(0)).try(&mut stdout);

        print(((size + 1023) / 1024).to_string().as_bytes(), &mut stdout);
        print(b"    ", &mut stdout);
        println(entry.as_bytes(), &mut stdout);
    }
}
fn main() {
    if let Some(ref x) = env::args().nth(1) {
        print_path(x);
    } else {
        print_path(".");
    }
}
