extern crate anyhow;
extern crate arg_parser;
#[cfg(target_os = "redox")]
extern crate libredox;

use std::io::{self, Write};
use anyhow::{Context, Result};
use arg_parser::ArgParser;
use std::fmt::Write as FmtWrite;
use std::env;
#[cfg(not(target_os = "redox"))]
use std::fs;

const MAN_PAGE: &'static str = /* @MANSTART{uptime} */ r#"
NAME
    uptime - show how long the system has been running

SYNOPSIS
    uptime [ -h | --help] [offset]

DESCRIPTION
    Prints the length of time the system has been up.

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

#[cfg(target_os = "redox")]
fn uptime() -> Result<i64> {
    let ts = libredox::call::clock_gettime(libredox::flag::CLOCK_MONOTONIC)?;
    Ok(ts.tv_sec)
}

#[cfg(target_os = "linux")]
fn uptime() -> Result<i64> {
    Ok(fs::read_to_string("/proc/uptime")?.split('.').nth(0).unwrap().parse()?)
}

fn main() -> Result<()> {
   let stdout = io::stdout();
   let mut stdout = stdout.lock();

   let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
   parser.parse(env::args());

    if parser.found("help") {
        stdout.write(MAN_PAGE.as_bytes())?;
        stdout.flush()?;
        return Ok(());
    }

    let uptime = uptime()?;
    let uptime_secs = uptime % 60;
    let uptime_mins = (uptime / SECONDS_PER_MINUTE) % 60;
    let uptime_hours = (uptime / SECONDS_PER_HOUR) % 24;
    let uptime_days = uptime / SECONDS_PER_DAY;

    let mut uptime_str = String::new();
    let fmt_result;
    if uptime_days > 0 {
        fmt_result = write!(&mut uptime_str, "{}d {}h {}m {}s\n", uptime_days,
                            uptime_hours, uptime_mins, uptime_secs);
    } else if uptime_hours > 0 {
        fmt_result = write!(&mut uptime_str, "{}h {}m {}s\n", uptime_hours,
                            uptime_mins, uptime_secs);
    } else if uptime_mins > 0 {
        fmt_result = write!(&mut uptime_str, "{}m {}s\n", uptime_mins,
                            uptime_secs);
    } else {
        fmt_result = write!(&mut uptime_str, "{}s\n", uptime_secs);
    }

    fmt_result.context("failed to parse uptime")?;
    stdout.write_all(uptime_str.as_bytes())?;
    Ok(())
}
