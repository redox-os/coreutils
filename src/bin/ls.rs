#![deny(warnings)]
extern crate chrono;

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::fs::FileType;
use std::path::Path;
use std::io::{stdout, StdoutLock, stderr, Stderr, Write};
use std::os::unix::fs::MetadataExt;

use std::process::exit;

use coreutils::{ArgParser, to_human_readable_string, system_time_to_utc_string};
use extra::option::OptionalExt;


const MAN_PAGE: &'static str = /* @MANSTART{ls} */ r#"
NAME
    ls - list directory contents

SYNOPSIS
    ls [ -h | --help | -l ][FILE]...

DESCRIPTION
    List information about the FILE(s), or the current directory

OPTIONS
    -a, --all
        do not ignore entries starting with .
    -h, --human-readable
        with -l, print human readable sizes
    --help
        display this help and exit
    -l
        use a long listing format
    -r, --reverse
        reverse order while sorting
    -R, --recursive
        list subdirectories recursively
    --mdate --modified-date
        display date of last modification
    --adate --accessed-date
        display date of last access
    --cdate --created-date
        display date of creation

"#; /* @MANEND */

fn mode_to_human_readable(file_type: &FileType, symlink_file_type: &FileType, mode: u32) -> String {

    let mut result = String::from("");
    if symlink_file_type.is_symlink() {
        result.push('l')
    } else if file_type.is_dir() {
        result.push('d');
    } else {
        result.push('-');
    }

    let mode_str = format!("{:>6o}", mode);
    let mode_chars = mode_str[3..].chars();
    for i in mode_chars {
        match i {
            '7' => result.push_str("rwx"),
            '6' => result.push_str("rw-"),
            '5' => result.push_str("r-x"),
            '4' => result.push_str("r--"),
            '3' => result.push_str("-wx"),
            '2' => result.push_str("-w-"),
            '1' => result.push_str("--x"),
            _ => result.push_str("---"),
        }
    }

    return result;
}

fn print_item(item_path: &str, parser: &ArgParser, stdout: &mut StdoutLock, stderr: &mut Stderr) {

    let mut link_error = "";
    let symlink_metadata = fs::symlink_metadata(&item_path).try(stderr);
    let metadata = match fs::metadata(&item_path) {
        Ok(metadata) => metadata,
        Err(_) => {
            link_error = "broken link";
            fs::symlink_metadata(&item_path).try(stderr)
        }
    };
    if parser.found("long-format") {
        stdout.write(&format!("{} {:>5} {:>5} ",
                              mode_to_human_readable(&(metadata.file_type()), &(symlink_metadata.file_type()), metadata.mode()),
                              metadata.uid(),
                              metadata.gid())
                              .as_bytes())
            .try(stderr);
        if parser.found("human-readable") {
            stdout.write(&format!("{:>6} ", to_human_readable_string(metadata.size())).as_bytes()).try(stderr);
        } else {
            stdout.write(&format!("{:>8} ", metadata.size()).as_bytes()).try(stderr);
        }
    }
    if parser.found("modified-date") || parser.found("long-format") {
        stdout.write(&format!("{:>20} ", system_time_to_utc_string(metadata.modified().expect("can't get modification date from file metadata"))).as_bytes()).try(stderr);
    }
    if parser.found("created-date") {
        stdout.write(&format!("{:>20} ", system_time_to_utc_string(metadata.created().expect("can't get creation date from file metadata"))).as_bytes()).try(stderr);
    }
    if parser.found("accessed-date") {
        stdout.write(&format!("{:>20} ", system_time_to_utc_string(metadata.accessed().expect("can't get access date from file metadata"))).as_bytes()).try(stderr);
    }

    if item_path.starts_with("./") {
        stdout.write(&item_path[2..].as_bytes()).try(stderr);
    } else {
        stdout.write(item_path.as_bytes()).try(stderr);
    }
    if parser.found("long-format") && symlink_metadata.file_type().is_symlink() {
        let symlink_target = fs::read_link(item_path)
            .expect("can't read link")
            .into_os_string()
            .into_string()
            .expect("c'ant get path as string");
        stdout.write(&format!(" -> {}", symlink_target).as_bytes()).try(stderr);
        if !link_error.is_empty() {
            stdout.write(&format!(" ({})", link_error).as_bytes()).try(stderr);
        }
    }

    stdout.write("\n".as_bytes()).try(stderr);
    stdout.flush().try(stderr);
}

fn list_dir(path: &str, parser: &ArgParser, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    let mut show_hidden = false;
    if parser.found("all") {
        show_hidden = true;
    }

    let metadata = fs::metadata(path).try(stderr);
    if metadata.is_dir() {
        let read_dir = Path::new(path).read_dir().try(stderr);

        let mut entries: Vec<String> = read_dir.filter_map(|x| x.ok())
            .map(|x| {
                     let file_name = x.file_name().to_string_lossy().into_owned();
                     file_name
                 })
            .filter(|x| match show_hidden {
                        true => true,
                        false => !x.starts_with("."),
                    })
            .collect();

        if parser.found("reverse") {
            entries.sort_by(|a, b| b.cmp(a));
        } else {
            entries.sort_by(|a, b| a.cmp(b));
        }

        for entry in entries.iter() {
            let mut entry_path = path.to_owned();
            if !entry_path.ends_with('/') {
                entry_path.push('/');
            }
            entry_path.push_str(&entry);
            print_item(&entry_path, &parser, stdout, stderr);
            if parser.found("recursive") && metadata.is_dir() {
                list_dir(&entry_path, parser, stdout, stderr);
            }
        }
    } else {
        print_item(&path, &parser, stdout, stderr);
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut parser = ArgParser::new(6)
        .add_flag(&["a", "all"])
        .add_flag(&["l", "long-format"])
        .add_flag(&["h", "human-readable"])
        .add_flag(&["r", "reverse"])
        .add_flag(&["R", "recursive"])
        .add_flag(&["mdate", "modified-date"])
        .add_flag(&["adate", "accessed-date"])
        .add_flag(&["cdate", "created-date"])
        .add_flag(&["", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }


    if parser.args.is_empty() {
        list_dir(".", &parser, &mut stdout, &mut stderr);
    } else {
        for dir in parser.args.iter() {
            list_dir(&dir, &parser, &mut stdout, &mut stderr);
        }
    }
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

#[test]
fn test_system_time_to_utc_string() {
    use std::time::SystemTime;
    use chrono::prelude::{DateTime,UTC};

    let system_time_now = SystemTime::now();
    let utc_now: DateTime<UTC> = UTC::now();
    assert_eq!(system_time_to_utc_string(system_time_now), utc_now.format("%Y-%m-%d %H:%M:%S").to_string());
}
