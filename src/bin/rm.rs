use std::env;
use std::fs;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("rm: no arguments");
        process::exit(1);
    }

    for ref path in env::args().skip(1) {
        if let Err(err) = fs::remove_file(path) {
            println!("rm: cannot remove '{}': {}", path, err);
        }
    }
}
