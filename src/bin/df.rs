#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;
#[cfg(target_os = "redox")]
extern crate syscall;

#[cfg(target_os = "redox")]
fn df(path: &str, parser: &arg_parser::ArgParser) -> ::std::io::Result<()> {
    use coreutils::to_human_readable_string;
    use std::io::Error;
    use syscall::data::StatVfs;

    let mut stat = StatVfs::default();
    {
        let fd = syscall::open(path, syscall::O_STAT).map_err(|err| Error::from_raw_os_error(err.errno))?;
        syscall::fstatvfs(fd, &mut stat).map_err(|err| Error::from_raw_os_error(err.errno))?;
        let _ = syscall::close(fd);
    }

    let size = stat.f_blocks * stat.f_bsize as u64;
    let used = (stat.f_blocks - stat.f_bfree) * stat.f_bsize as u64;
    let free = stat.f_bavail * stat.f_bsize as u64;
    let percent = (100.0 * used as f64 / size as f64) as u64;

    if parser.found("human-readable") {
        println!("{:<10}{:>10}{:>10}{:>10}{:>5}",
                 path,
                 to_human_readable_string(size),
                 to_human_readable_string(used),
                 to_human_readable_string(free),
                 format!("{}%", percent));
    } else {
        println!("{:<10}{:>10}{:>10}{:>10}{:>5}",
                 path,
                 (size + 1023)/1024,
                 (used + 1023)/1024,
                 (free + 1023)/1024,
                 format!("{}%", percent));
    }

    Ok(())
}

#[cfg(target_os = "redox")]
fn main() {
    use std::env;
    use std::fs::File;
    use std::io::{stdout, stderr, BufRead, BufReader, Write};
    use std::process::exit;
    use arg_parser::ArgParser;
    #[macro_use]
    use coreutils::arg_parser::ArgParserExt;
    use extra::option::OptionalExt;

    const MAN_PAGE: &'static str = /* @MANSTART{df} */ r#"
    NAME
        df - view filesystem space usage

    SYNOPSIS
        df [ -h | --help ] FILE...

    DESCRIPTION
        df gets the filesystem space usage for the filesystem providing FILE

    OPTIONS
        -h
        --human-readable
            human readable output
        --help
            display this help and exit
    "#; /* @MANEND */

    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "human-readable"])
        .add_flag(&["help"]);
    parser.process_common(help_info!("df"), MAN_PAGE);

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    println!("{:<10}{:>10}{:>10}{:>10}{:>5}", "Path", "Size", "Used", "Free", "Use%");
    if parser.args.is_empty() {
        let file = BufReader::new(File::open("sys:scheme").try(&mut stderr));
        for line in file.lines() {
            let _ = df(&format!("{}:", line.try(&mut stderr)), &parser);
        }
    } else {
        for path in &parser.args {
            df(&path, &parser).try(&mut stderr);
        }
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
