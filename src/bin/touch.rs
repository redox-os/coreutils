use std::env;
use std::fs::File;
use std::process;

fn main() {
    // TODO support clap

    if env::args().count() < 2 {
        println!("No arguments provided!");
        process::exit(1);
    }

    // TODO update file modification date/time

    for arg in env::args().skip(1) {
        if let Err(_) = File::open(&arg) {
            File::create(&arg).unwrap();
        }
    }
}
