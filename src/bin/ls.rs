#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};

use coreutils::extra::OptionalExt;

fn print_path(path: &str) {
    let mut entries = Vec::new();

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stderr = stderr();
    let mut stderr = stderr.lock();

    let dir = fs::read_dir(path).try(&mut stderr);

    for entry_result in dir {
        let entry = entry_result.try(&mut stderr);
        let directory = entry.file_type().map(|x| x.is_dir()).unwrap_or(false);

        let file_name = entry.file_name();
        let path_str = file_name.to_str().try(&mut stderr);
        entries.push(path_str.to_string());

        if directory {
            entries.last_mut().unwrap().push('/');
        }
    }

    entries.sort();

    for entry in entries {
        stdout.write(entry.as_bytes()).try(&mut stderr);
    }
}

fn main() {
    let path = env::args().nth(1);

    if let Some(ref x) = path {
        print_path(x);
    } else {
        print_path(".");
    } // dafuq borrowck. Really you needa do deref coercions better.
}
