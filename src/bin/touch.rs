use std::env;
use std::fs::File;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("No arguments provided!");
        process::exit(1);
    }

    for arg in env::args().skip(1) {
        if let Err(_) = File::open(arg.to_string()) {
            File::create(arg).unwrap();
        }
    }
}
