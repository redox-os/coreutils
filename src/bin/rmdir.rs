use std::env;
use std::fs;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("rmdir: no arguments");
        process::exit(1);
    }

    for path in env::args().skip(1) {
        if let Err(err) = fs::remove_dir(path) {
            println!("rmdir: cannot remove '{}': {}", path, err);
        }
    }
}
