extern crate anyhow;
extern crate arg_parser;
#[cfg(target_os = "redox")]
extern crate libredox;

use anyhow::Result;

#[cfg(target_os = "redox")]
fn main() -> Result<()> {
    use std::env;
    use std::io::{stdout, Write};
    use anyhow::{bail, Context};
    use arg_parser::ArgParser;

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
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes())?;
        stdout.flush()?;
        return Ok(());
    }

    let Some(sig_arg) = parser.args.get(0) else {
        bail!("No signal. Use --help to see the usage.");
    };
    let sig = sig_arg.parse::<u32>().context("Failed to parse signal")?;
    if sig > 0x7F {
        bail!("Signal greater than 127. Use --help to see the usage.");
    }
    if parser.args.is_empty() {
        bail!("No pids. Use --help to see the usage.");
    }

    for pid_str in &parser.args[1..] {
        let pid = pid_str.parse::<usize>()?;
        libredox::call::kill(pid, sig)?;
    }
    Ok(())
}

#[cfg(not(target_os = "redox"))]
fn main() -> Result<()> {
    Err(anyhow::anyhow!("error: unimplemented outside redox"))
}
