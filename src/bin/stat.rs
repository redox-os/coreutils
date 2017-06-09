#![deny(warnings)]

extern crate coreutils;
extern crate extra;
extern crate syscall;

use std::env;
use std::io::{stdout, stderr, Write};
use coreutils::ArgParser;
use extra::option::OptionalExt;

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

    for path in &parser.args[0..] {
        let mut st = syscall::Stat::default();
        let fd = syscall::open(path, syscall::O_CLOEXEC | syscall::O_STAT).unwrap();
        syscall::fstat(fd, &mut st).unwrap();
        syscall::close(fd).unwrap();
        println!("File: {}", path);
        println!("Size: {}  Blocks: {}  IO Block: {}", st.st_size, st.st_blocks, st.st_blksize);
        println!("Device: {}  Inode: {}  Links: {}", st.st_dev, st.st_ino, st.st_nlink);
        println!("Access: {:o}  Uid: {}  Gid: {}", st.st_mode, st.st_uid, st.st_gid);
        println!("Access: {}.{:09}", st.st_atime, st.st_atime_nsec);
        println!("Modify: {}.{:09}", st.st_mtime, st.st_mtime_nsec);
        println!("Change: {}.{:09}", st.st_ctime, st.st_ctime_nsec);
    }
}
