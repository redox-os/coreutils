#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate time;
extern crate userutils;

use std::{env, fmt, fs};
use std::fs::File;
use std::io::{stdout, stderr, Read, Write};
use std::vec::Vec;
use arg_parser::ArgParser;
use extra::option::OptionalExt;
use userutils::{Passwd, Group};
use std::os::unix::fs::MetadataExt;
use time::Timespec;

const MAN_PAGE: &'static str = /* @MANSTART{stat} */ r#"
NAME
    stat - display file status

SYNOPSIS
    stat [ -h | --help ] FILE...

DESCRIPTION
    Displays file status.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

const TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S.%f %z";

struct Perms(u32);

impl fmt::Display for Perms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(0{:o}/", self.0 & 0o777)?;
        let perm = |i, c| {
            if self.0 & ((1 << i) as u32) != 0 {
                c
            } else {
                "-"
            }
        };
        write!(f, "{}{}{}", perm(8, "r"), perm(7, "w"), perm(6, "x"))?;
        write!(f, "{}{}{}", perm(5, "r"), perm(4, "w"), perm(3, "x"))?;
        write!(f, "{}{}{}", perm(2, "r"), perm(1, "w"), perm(0, "x"))?;
        write!(f, ")")?;
        Ok(())
    }
}

fn lookup_user<'a>(passwd: &'a [Passwd], uid: u32) -> &'a str {
    for i in passwd {
        if i.uid == uid {
            return i.user;
        }
    }
    "UNKNOWN"
}

fn lookup_group<'a>(group: &'a [Group], gid: u32) -> &'a str {
    for i in group {
        if i.gid == gid {
            return i.group;
        }
    }
    "UNKNOWN"
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    let mut passwd_string = String::new();
    File::open("/etc/passwd").unwrap().read_to_string(&mut passwd_string).unwrap();

    let mut passwd = Vec::new();
    for line in passwd_string.lines() {
        if let Ok(entry) = Passwd::parse(line) {
            passwd.push(entry);
        }
    }

    let mut group_string = String::new();
    File::open("/etc/group").unwrap().read_to_string(&mut group_string).unwrap();

    let mut groups = Vec::new();
    for line in group_string.lines() {
        if let Ok(entry) = Group::parse(line) {
            groups.push(entry);
        }
    }

    for path in &parser.args[0..] {
        let meta = fs::symlink_metadata(path).unwrap();
        let file_type = if meta.file_type().is_symlink() {
            "symbolic link"
        } else if meta.is_file() {
            "regular file"
        } else if meta.is_dir() {
            "directory"
        } else {
            ""
        };
        if meta.file_type().is_symlink() {
            println!("File: {} -> {}", path, fs::read_link(path).unwrap().display());
        } else {
            println!("File: {}", path);
        }
        println!("Size: {}  Blocks: {}  IO Block: {} {}", meta.size(), meta.blocks(), meta.blksize(), file_type);
        println!("Device: {}  Inode: {}  Links: {}", meta.dev(), meta.ino(), meta.nlink());
        println!("Access: {}  Uid: ({}/ {})  Gid: ({}/ {})", Perms(meta.mode()),
                                                             meta.uid(), lookup_user(&passwd, meta.uid()),
                                                             meta.gid(), lookup_group(&groups, meta.gid()));
        println!("Access: {}", time::at(Timespec::new(meta.atime(), meta.atime_nsec() as i32)).strftime(TIME_FMT).unwrap());
        println!("Modify: {}", time::at(Timespec::new(meta.mtime(), meta.mtime_nsec() as i32)).strftime(TIME_FMT).unwrap());
        println!("Change: {}", time::at(Timespec::new(meta.ctime(), meta.ctime_nsec() as i32)).strftime(TIME_FMT).unwrap());
    }
}
