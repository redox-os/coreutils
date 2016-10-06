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
    ls [ -h | --help | -l ][FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -h
        with -l, print human readable sizes
    --help
        display this help and exit
    -l
        use a long listing format
"#; /* @MANEND */

fn list_dir(path: &str, long_format: bool, human_readable: bool, string: &mut String, stderr: &mut Stderr) {
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
                let mut entry_path = path.to_owned();
                if !entry_path.ends_with('/') {
                    entry_path.push('/');
                }
                entry_path.push_str(&entry);

                let metadata = fs::metadata(entry_path).try(stderr);
                if human_readable {
                    string.push_str(&format!("{:>7o} {:>5} {:>5} {} ",
                                             metadata.mode(),
                                             metadata.uid(),
                                             metadata.gid(),
                                             to_human_readable_string(metadata.size())));
                } else {
                    string.push_str(&format!("{:>7o} {:>5} {:>5} {:>8} ",
                                             metadata.mode(),
                                             metadata.uid(),
                                             metadata.gid(),
                                             metadata.size()));
                }
            }
            string.push_str(entry);
            string.push('\n');
        }
    } else {
        if long_format {
            if human_readable {
                string.push_str(&format!("{:>7o} {:>5} {:>5} {} ",
                                         metadata.mode(),
                                         metadata.uid(),
                                         metadata.gid(),
                                         to_human_readable_string(metadata.size())));
            } else {
                string.push_str(&format!("{:>7o} {:>5} {:>5} {:>8} ",
                                         metadata.mode(),
                                         metadata.uid(),
                                         metadata.gid(),
                                         metadata.size()));
            }
        }
        string.push_str(path);
        string.push('\n');
    }
}

fn to_human_readable_string(size: u64) -> String {
    if size < 1024 {
        return format!("{}", size);
    }

    static UNITS: [&'static str; 7] = ["", "K", "M", "G", "T", "P", "E"];

    let digit_groups = ((64 - size.leading_zeros()) / 10) as i32;
    format!("{:.1}{}",
            size as f64 / 1024f64.powi(digit_groups),
            UNITS[digit_groups as usize])
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut long_format = false;
    let mut human_readable = false;
    let mut args = Vec::new();
    for arg in env::args().skip(1) {
        if arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        } else if arg.as_str() == "-l" {
            long_format = true;
        } else if arg.as_str() == "-h" {
            human_readable = true;
        } else {
            args.push(arg);
        }
    }

    let mut string = String::new();
    if args.is_empty() {
        list_dir(".", long_format, human_readable, &mut string, &mut stderr);
    } else {
        for dir in args {
            list_dir(&dir, long_format, human_readable, &mut string, &mut stderr);
        }
    }
    stdout.write(string.as_bytes()).try(&mut stderr);
}

#[test]
fn test_human_readable() {
    assert_eq!(to_human_readable_string(0), "0");
    assert_eq!(to_human_readable_string(1023), "1023");
    assert_eq!(to_human_readable_string(1024), "1.0K");
    assert_eq!(to_human_readable_string(1024 + 100), "1.1K");
    assert_eq!(to_human_readable_string(1024u64.pow(2) * 2), "2.0M");
    assert_eq!(to_human_readable_string(1024u64.pow(3) * 3), "3.0G");
    assert_eq!(to_human_readable_string(1024u64.pow(4) * 4), "4.0T");
    assert_eq!(to_human_readable_string(1024u64.pow(5) * 5), "5.0P");
    assert_eq!(to_human_readable_string(1024u64.pow(6) * 6), "6.0E");
}
