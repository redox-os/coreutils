#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::io::{stdout, stderr, StdoutLock, Stderr, Write};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory
"#; /* @MANEND */

fn list_entry(name: &str, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    stdout.write(name.as_bytes()).try(stderr);
    stdout.write(b"\n").try(stderr);
}

fn list_dir(path: &str, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    if fs::metadata(path).try(stderr).is_dir() {
        let dir = fs::read_dir(path).try(stderr);

        let mut entries = Vec::new();
        for entry_result in dir {
            let entry = entry_result.try(stderr);
            let directory = entry.file_type().map(|x| x.is_dir()).unwrap_or(false);

            let file_name = entry.file_name();
            let path_str = file_name.to_str().try(stderr);
            entries.push(path_str.to_owned());

            if directory {
                entries.last_mut().unwrap().push('/');
            }
        }

        entries.sort();

        for entry in entries {
            list_entry(&entry, stdout, stderr);
        }
    } else {
        list_entry(path, stdout, stderr);
    }
}
fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut args = env::args().skip(1);
    if let Some(ref x) = args.next() {
        list_dir(x, &mut stdout, &mut stderr);
        for y in args {
            list_dir(&y, &mut stdout, &mut stderr);
        }
    } else {
        list_dir(".", &mut stdout, &mut stderr);
    }
}
