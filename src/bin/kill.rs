#![deny(warnings)]

extern crate coreutils;
extern crate extra;
#[cfg(target_os = "redox")]
extern crate syscall;

#[cfg(target_os = "redox")]
fn main() {
    use std::env;
    use std::io::{stdout, stderr, Error, Write};
    use coreutils::ArgParser;
    use extra::io::fail;
    use extra::option::OptionalExt;

    const MAN_PAGE: &'static str = /* @MANSTART{kill} */ r#"
    NAME
        kill - send a signal

    SYNOPSIS
        kill [ -h | --help ] MODE PID...

    DESCRIPTION
        The kill utility sends a signal to a process. Multiple PIDs can be passed.

    OPTIONS
        --help, -h
            print this message
    "#; /* @MANEND */

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

    if let Some(sig_arg) = parser.args.get(0) {
        let sig = sig_arg.parse::<usize>().try(&mut stderr);
        if sig <= 0x7F {
            if parser.args.is_empty() {
                fail("No pids. Use --help to see the usage.", &mut stderr);
            }

            for pid_str in &parser.args[1..] {
                let pid = pid_str.parse::<usize>().try(&mut stderr);
                syscall::kill(pid, sig).map_err(|err| Error::from_raw_os_error(err.errno)).try(&mut stderr);
            }
        } else {
            fail("Signal greater than 127. Use --help to see the usage.", &mut stderr);
        }
    } else {
        fail("No signal. Use --help to see the usage.", &mut stderr);
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
