#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;
#[cfg(target_os = "redox")]
extern crate syscall;

#[cfg(target_os = "redox")]
fn free(parser: &arg_parser::ArgParser) -> ::std::io::Result<()> {
    use coreutils::to_human_readable_string;
    use std::io::Error;
    use syscall::data::StatVfs;

    let mut stat = StatVfs::default();
    {
        let fd = syscall::open("memory:", syscall::O_STAT).map_err(|err| Error::from_raw_os_error(err.errno))?;
        syscall::fstatvfs(fd, &mut stat).map_err(|err| Error::from_raw_os_error(err.errno))?;
        let _ = syscall::close(fd);
    }

    let size = stat.f_blocks * stat.f_bsize as u64;
    let used = (stat.f_blocks - stat.f_bfree) * stat.f_bsize as u64;
    let free = stat.f_bavail * stat.f_bsize as u64;

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
fn main() {
    use std::io::{stdout, stderr, Write};
    use std::process::exit;
    use arg_parser::ArgParser;
    #[macro_use]
    use coreutils::arg_parser::ArgParserExt;
    use extra::option::OptionalExt;

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

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "human-readable"])
        .add_flag(&["help"]);
    parser.process_common(help_info!("free"), MAN_PAGE);

    println!("{:<8}{:>10}{:>10}{:>10}", "", "total", "used", "free");
    free(&parser).try(&mut stderr);
}

#[cfg(not(target_os = "redox"))]
fn main() {
    use std::io::{stderr, Write};
    use std::process::exit;

    let mut stderr = stderr();
    stderr.write(b"error: unimplemented outside redox").unwrap();
    exit(1);
}
