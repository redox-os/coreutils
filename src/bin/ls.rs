extern crate coreutils;

use std::env;
use std::fs;
use std::io::{Write, stdout};

use coreutils::extra::OptionalExt;

fn print(path: &str) {
    let mut entries = Vec::new();

    let dir = fs::read_dir(path).try();

    for entry_result in dir {
        let entry = entry_result.try();
        let directory = entry.file_type().map(|x| x.is_dir()).unwrap_or(false);

        let file_name = entry.file_name();
        let path_str = file_name.to_str().try();
        entries.push(path_str.to_string());

        if directory {
            entries.last_mut().unwrap().push('/');
        }
    }

    entries.sort();

    let mut stdout = stdout();

    for entry in entries {
        stdout.write(entry.as_bytes()).try();
        stdout.write(b"\n").try();
    }
}

fn main() {
    let path = env::args().nth(1);

    if let Some(ref x) = path {
        print(x);
    } else {
        print(".");
    } // dafuq borrowck. Really you needa do deref coercions better.
}
