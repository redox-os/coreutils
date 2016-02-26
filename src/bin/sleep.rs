#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::thread;
use std::io;

use coreutils::extra::{OptionalExt, fail};

#[allow(deprecated)]
fn main() {
    let mut stdout = io::stdout();

    if let Some(arg) = env::args().nth(1) {
        // TODO: (not supported in Redox due to missing _mulodi4) thread::sleep(Duration::new(arg.parse().try(), 0))
        thread::sleep_ms(arg.parse::<u32>().try(&mut stdout) * 1000);
    } else {
        let stdout = io::stdout();
        fail("missing argument.", &mut stdout.lock());
    }
}
