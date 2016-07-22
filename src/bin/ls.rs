#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::path::Path;
use std::io::{stdout, stderr, StdoutLock, Stderr, Write};
use extra::option::OptionalExt;
use std::process::exit;

/* CONST
 */

const OPTION_NOT_FOUND: &'static str = r#"
Option not found.
Please to use help option (-h/--help) to see available options.
"#;

const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [ -h --help | -m --mode | -s --size ] [FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -h
    --help
        display this help and exit

    -m
    --mode
        display permissions file

    -s
    --size
        display the size (bytes) of each file
"#;

/* STRUCT
 */

struct Flags {
    mode: bool,
    size: bool,
}

/* FUNCTIONS
 */

fn list_entry(name: &str, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    stdout.write(name.as_bytes()).try(stderr);
    stdout.write(b"\n").try(stderr);
}

fn list_dir(path: &str, flags_struct: &Flags, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    if fs::metadata(path).try(stderr).is_dir() {
        let read_dir = Path::new(path).read_dir().try(stderr);
        let mut entries = vec![];
        for dir in read_dir {
            let dir = match dir {
                Ok(x) => x,
                Err(_) => continue,
            };
            let mut to_return = "".to_string();
            to_return.push_str(dir.file_name().to_string_lossy().into_owned().as_str());
            if dir.file_type().try(stderr).is_dir() {
                to_return.push('/');
            }
            to_return.push('\t');
            let metadata = dir.metadata().unwrap();
            if flags_struct.mode {
                to_return.push_str(metadata.mode().to_string().as_str());
                to_return.push('\t');
            }
            if flags_struct.size {
                to_return.push_str(metadata.len().to_string().as_str());
                to_return.push('\t');
            }
            entries.push(to_return);
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

    let mut path_file = ".".to_string();

    let mut flags_struct = Flags {
        mode : false,
        size : false,
    };

    for arg in env::args().skip(1){
        match arg.as_str() {
            "-h" | "--help" => {
                stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
                stdout.flush().try(&mut stderr);
                exit(0);    
            },
            "-m" | "--mode" => flags_struct.mode = true,
            "-s" | "--size" => flags_struct.size = true,
            e @ _ => {
                if ! e.starts_with("-") {
                    path_file = e.to_string();
                } else {
                    stdout.write(OPTION_NOT_FOUND.as_bytes()).try(&mut stderr);
                    stdout.flush().try(&mut stderr);
                    exit(0);
                }
            },
        }
    }

    list_dir(path_file.as_str(), &flags_struct, &mut stdout, &mut stderr);
}
