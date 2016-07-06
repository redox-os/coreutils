#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::path::Path;
use std::io::{stdout, stderr, StdoutLock, Stderr, Write};
use extra::option::OptionalExt;
use std::process::exit;

const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [ -h | --help ][FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn list_entry(name: &str, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    stdout.write(name.as_bytes()).try(stderr);
    stdout.write(b"\n").try(stderr);
}

fn list_dir(path: &str, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    if fs::metadata(path).try(stderr).is_dir() {
        let read_dir = Path::new(path).read_dir().try(stderr);

        let mut entries = vec![];
        for dir in read_dir {
            let dir = match dir {
                Ok(x) => x,
                Err(_) => continue,
            };
            let mut file_name = dir.file_name().to_string_lossy().into_owned();
            if dir.file_type().try(stderr).is_dir() {
                file_name.push('/');
            }
            entries.push(file_name);
        }

        entries.sort();

        for entry in entries.iter() {
            list_entry(entry, stdout, stderr);
        }
    } else {
        list_entry(path, stdout, stderr);
    }
}
fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for arg in env::args().skip(1){
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        }
    }

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
