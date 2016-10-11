#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::path::Path;
use std::io::{stdout, stderr, StdoutLock, Stderr, Write};
use std::os::unix::fs::MetadataExt;
use std::process::exit;

use coreutils::{ArgParser, Flag, to_human_readable_string};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [ -h | --help | -l ][FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -h
    --human-readable
        with -l, print human readable sizes
    --help
        display this help and exit
    -l
        use a long listing format
"#; /* @MANEND */

fn list_dir(path: &str, parser: &ArgParser, string: &mut String, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    if parser.enabled_flag(Flag::Long("help")) {
        stdout.write(MAN_PAGE.as_bytes()).try(stderr);
        stdout.flush().try(stderr);
        exit(0);
    }

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
            if parser.enabled_flag(Flag::Short('l')) {
                let mut entry_path = path.to_owned();
                if !entry_path.ends_with('/') {
                    entry_path.push('/');
                }
                entry_path.push_str(&entry);

                let metadata = fs::metadata(entry_path).try(stderr);
                string.push_str(&format!("{:>7o} {:>5} {:>5} ",
                                         metadata.mode(),
                                         metadata.uid(),
                                         metadata.gid()));
                if parser.enabled_flag(Flag::Long("human-readable")) {
                    string.push_str(&format!("{:>6} ", to_human_readable_string(metadata.size())));
                } else {
                    string.push_str(&format!("{:>8} ", metadata.size()));
                }
            }
            string.push_str(entry);
            string.push('\n');
        }
    } else {
        if parser.enabled_flag(Flag::Short('l')) {
            string.push_str(&format!("{:>7o} {:>5} {:>5} ",
                                     metadata.mode(),
                                     metadata.uid(),
                                     metadata.gid()));
            if parser.enabled_flag(Flag::Long("human-readable")) {
                 string.push_str(&format!("{:>6} ", to_human_readable_string(metadata.size())));
             } else {
                 string.push_str(&format!("{:>8} ", metadata.size()));
             }
        }
        string.push_str(path);
        string.push('\n');
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(3)
        .add_flag("l", "long-format")
        .add_flag("h", "human-readable")
        .add_flag("", "help");
    parser.initialize(env::args());

    let mut string = String::new();
    if parser.args.is_empty() {
        list_dir(".", &parser, &mut string, &mut stdout, &mut stderr);
    } else {
        for dir in parser.args.iter() {
            list_dir(&dir, &parser, &mut string, &mut stdout, &mut stderr);
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
