extern crate anyhow;
extern crate arg_parser;
extern crate coreutils;
#[cfg(target_os = "redox")]
extern crate libredox;

use anyhow::Result;

#[cfg(target_os = "redox")]
fn free(parser: &arg_parser::ArgParser) -> Result<()> {
    use anyhow::Context;
    use coreutils::to_human_readable_string;
    use libredox::{Fd, flag};

    let stat = Fd::open("/scheme/memory", flag::O_PATH, 0)
        .context("failed to open `/scheme/memory`")?
        .statvfs()
        .context("failed to fstatvfs `/scheme/memory`")?;

    let size = stat.f_blocks as u64 * stat.f_bsize as u64;
    let used = (stat.f_blocks as u64 - stat.f_bfree as u64) * stat.f_bsize as u64;
    let free = stat.f_bavail as u64 * stat.f_bsize as u64;

    if parser.found("human-readable") {
        println!("{:<8}{:>10}{:>10}{:>10}",
                 "Mem:",
                 to_human_readable_string(size),
                 to_human_readable_string(used),
                 to_human_readable_string(free));
    } else {
        println!("{:<8}{:>10}{:>10}{:>10}",
                 "Mem:",
                 (size + 1023)/1024,
                 (used + 1023)/1024,
                 (free + 1023)/1024);
    }

    Ok(())
}

#[cfg(target_os = "redox")]
fn main() -> Result<()> {
    use std::env;
    use std::io::{stdout, Write};
    use std::process::exit;
    use arg_parser::ArgParser;

    const MAN_PAGE: &'static str = /* @MANSTART{free} */ r#"
    NAME
        free - view memory usage

    SYNOPSIS
        free [ -h | --help ]

    DESCRIPTION
        free displays the current memory usage of the system

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

    println!("{:<8}{:>10}{:>10}{:>10}", "", "total", "used", "free");
    free(&parser)?;
    Ok(())
}

#[cfg(not(target_os = "redox"))]
fn main() -> Result<()> {
    Err(anyhow::anyhow!("error: unimplemented outside redox"))
}
