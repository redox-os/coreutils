#![deny(warnings)]

extern crate extra;

use std::env;
use std::fs;
use std::path::Path;
use std::io::{stdout, stderr, Stderr, Write};
use std::os::unix::fs::MetadataExt;
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

fn list_dir(path: &str, long_format: bool, string: &mut String, stderr: &mut Stderr) {
    let metadata = fs::metadata(path).try(stderr);
    if metadata.is_dir() {
        let read_dir = Path::new(path).read_dir().try(stderr);

        let mut entries: Vec<String> = read_dir.filter_map(|x| x.ok()).map(|dir| {
                let mut file_name = dir.file_name().to_string_lossy().into_owned();
                if dir.file_type().try(stderr).is_dir() {
                    file_name.push('/');
                }
                file_name
            })
            .collect();

        entries.sort();

        for entry in entries.iter() {
            if long_format {
                let entry_path = if path == "." || path == "./" {
                    entry.clone()
                } else {
                    let mut entry_path = path.to_owned();
                    if !entry_path.ends_with('/') {
                        entry_path.push('/');
                    }
                    entry_path.push_str(&entry);
                    entry_path
                };

                let metadata = fs::metadata(entry_path).try(stderr);
                string.push_str(&format!("{:>7o} {:>5} {:>5} {:>8} ", metadata.mode(), metadata.uid(), metadata.gid(), metadata.size()/1024));
            }
            string.push_str(entry);
            string.push('\n');
        }
    } else {
        if long_format {
            string.push_str(&format!("{:>7o} {:>5} {:>5} {:>8} ", metadata.mode(), metadata.uid(), metadata.gid(), metadata.size()/1024));
        }
        string.push_str(path);
        string.push('\n');
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut long_format = false;
    let mut args = Vec::new();
    for arg in env::args().skip(1) {
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        } else if arg.as_str() == "-l" {
            long_format = true;
        } else {
            args.push(arg);
        }
    }

    let mut string = String::new();
    if args.is_empty() {
        list_dir(".", long_format, &mut string, &mut stderr);
    } else {
        for dir in args {
            list_dir(&dir, long_format, &mut string, &mut stderr);
        }
    }
    stdout.write(string.as_bytes()).try(&mut stderr);
}
