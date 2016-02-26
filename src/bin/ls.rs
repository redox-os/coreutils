#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::stdout;

use coreutils::extra::{OptionalExt, println};

fn print_path(path: &str) {
    let mut entries = Vec::new();

    let stdout = stdout();
    let mut stdout = stdout.lock();

    let dir = fs::read_dir(path).try(&mut stdout);

    for entry_result in dir {
        let entry = entry_result.try(&mut stdout);
        let directory = entry.file_type().map(|x| x.is_dir()).unwrap_or(false);

        let file_name = entry.file_name();
        let path_str = file_name.to_str().try(&mut stdout);
        entries.push(path_str.to_string());

        if directory {
            entries.last_mut().unwrap().push('/');
        }
    }

    entries.sort();

    for entry in entries {
        println(entry.as_bytes(), &mut stdout);
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
