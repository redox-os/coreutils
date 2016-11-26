#![deny(warnings)]

extern crate coreutils;
extern crate extra;
#[cfg(target_os = "redox")]
extern crate syscall;

#[cfg(target_os = "redox")]
fn main() {
    use std::env;
    use std::io::{stdout, stderr, Error, Write};
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

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag("h", "help");
    parser.parse(env::args());

    if parser.found(&'h') || parser.found("help") {
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
            let fd = syscall::open(&path, syscall::O_STAT).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
            syscall::fstatvfs(fd, &mut stat).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
            let _ = syscall::close(fd);
        }

        let size = stat.f_blocks * stat.f_bsize as u64 / 1024;
        let used = (stat.f_blocks - stat.f_bfree) * stat.f_bsize as u64 / 1024;
        let free = stat.f_bavail * stat.f_bsize as u64 / 1024;
        let percent = (100.0 * used as f64 / size as f64) as u64;
        println!("{:<8}{:<8}{:<8}{:<5}", size, used, free, format!("{}%", percent));
    }
}

#[cfg(not(target_os = "redox"))]
fn main() {
    use std::io::{stderr, Write};
    use std::process::exit;

    let mut stderr = stderr();
    stderr.write(b"error: unimplemented outside redox").unwrap();
    exit(1);
}
