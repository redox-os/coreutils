//#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, Write};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory
"#; /* @MANEND */

fn print_path(path: &str) {
    let mut entries = Vec::new();

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let dir = fs::read_dir(path).try(&mut stderr);

    for entry_result in dir {
        let entry = entry_result.try(&mut stderr);
        let directory = entry.file_type().map(|x| x.is_dir()).unwrap_or(false);

        let file_name = entry.file_name();
        let path_str = file_name.to_str().try(&mut stderr);
        entries.push(path_str.to_owned());

        if directory {
            entries.last_mut().unwrap().push('/');
        }
    }

    entries.sort();

    for entry in entries {
        stdout.write(entry.as_bytes()).try(&mut stderr);
        stdout.write(b"\n").try(&mut stderr);
    }
}

fn main() {
    if let Some(ref x) = env::args().nth(1) {
        print_path(x);
    } else {
        print_path(".");
    }
}
