#![deny(warnings)]

extern crate extra;

use std::fs::File;
use std::io;

use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{ps} */ r#"
NAME
    ps - report a snapshot of the current processes

SYNOPSIS
    ps

DESCRIPTION
    Displays information about processes and threads that are currently active
"#; /* @MANEND */

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut file = File::open("context:").try(&mut stderr);
    io::copy(&mut file, &mut stdout).try(&mut stderr);
}
