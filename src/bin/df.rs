#![deny(warnings)]

extern crate coreutils;
extern crate extra;
extern crate syscall;

use std::env;
use std::fs::File;
use std::io::{stdout, stderr, Error, Write};
use std::os::unix::io::AsRawFd;
use std::process::exit;
use coreutils::ArgParser;
use extra::io::fail;
use extra::option::OptionalExt;
use syscall::data::StatVfs;

const MAN_PAGE: &'static str = /* @MANSTART{df} */ r#"
NAME
    df - view filesystem space usage

SYNOPSIS
    df [ -h | --help ] FILE...

DESCRIPTION
    df gets the filesystem space usage for the filesystem providing FILE

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.initialize(env::args());

    if parser.flagged(&'h') || parser.flagged("help") {
        stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    if parser.args.is_empty() {
        fail("No arguments. Use --help to see the usage.", &mut stderr);
    }
    
    println!("{:<8}{:<8}{:<8}{:<5}", "Size", "Used", "Free", "Use%");
    for path in &parser.args {
        let mut stat = StatVfs::default();
        {
            let file = File::open(&path).try(&mut stderr);
            syscall::fstatvfs(file.as_raw_fd(), &mut stat).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
        }

        let size = stat.f_blocks * stat.f_bsize as u64 / 1024;
        let used = (stat.f_blocks - stat.f_bfree) * stat.f_bsize as u64 / 1024;
        let free = stat.f_bavail * stat.f_bsize as u64 / 1024;
        let percent = (100.0 * used as f64 / size as f64) as u64;
        println!("{:<8}{:<8}{:<8}{:<5}", size, used, free, format!("{}%", percent));
    }
}
