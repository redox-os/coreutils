extern crate anyhow;
extern crate arg_parser;
extern crate coreutils;
#[cfg(target_os = "redox")]
extern crate libredox;

use anyhow::Result;

#[cfg(target_os = "redox")]
fn df(path: &str, parser: &arg_parser::ArgParser) -> Result<()> {
    use coreutils::to_human_readable_string;
    use libredox::{Fd, flag};
    use std::io::Error;

    let stat = Fd::open(path, flag::O_PATH, 0)?.statvfs()?;

    let size = stat.f_blocks as u64 * stat.f_bsize as u64;
    let used = (stat.f_blocks as u64 - stat.f_bfree as u64) * stat.f_bsize as u64;
    let free = stat.f_bavail as u64 * stat.f_bsize as u64;
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
fn main() -> Result<()> {
    use std::env;
    use std::fs::File;
    use std::io::{stdout, BufRead, BufReader, Write};
    use std::process::exit;
    use arg_parser::ArgParser;

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
    parser.parse(env::args());

    if parser.found("help") {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        stdout.write(MAN_PAGE.as_bytes())?;
        stdout.flush()?;
        exit(0);
    }

    println!("{:<10}{:>10}{:>10}{:>10}{:>5}", "Path", "Size", "Used", "Free", "Use%");
    if parser.args.is_empty() {
        let file = BufReader::new(File::open("sys:scheme")?);
        for line in file.lines() {
            let _ = df(&format!("{}:", line?), &parser);
        }
    } else {
        for path in &parser.args {
            df(&path, &parser)?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "redox"))]
fn main() -> Result<()> {
    Err(anyhow::anyhow!("error: unimplemented outside redox"))
}
