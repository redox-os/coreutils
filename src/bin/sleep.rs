extern crate coreutils;

use std::env;
use std::time::Duration;
use std::thread;

use coreutils::extra::{OptionalExt, fail};

fn main() {
    if let Some(arg) = env::args().nth(1) {
        //TODO: (not supported in Redox due to missing _mulodi4) thread::sleep(Duration::new(arg.parse().try(), 0))
        thread::sleep_ms(arg.parse::<u32>().try() * 1000);
    } else {
        fail("missing argument.");
    }
}
