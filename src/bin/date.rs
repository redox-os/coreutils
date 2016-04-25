#![deny(warnings)]

extern crate extra;

use std::io::{stdout, stderr, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use extra::option::OptionalExt;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let time = SystemTime::now();
    let duration = time.duration_since(UNIX_EPOCH).try(&mut stderr);

    stdout.write(&format!("{}\n", duration.as_secs()).as_bytes()).try(&mut stderr);
}
