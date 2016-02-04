use std::env;
use std::fs;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("realpath: no arguments");
        process::exit(1);
    }

    for ref path in env::args().skip(1) {
        match fs::canonicalize(path) {
            Ok(realpath) => println!("{}", realpath.display()),
            Err(err) => println!("realpath: cannot get path of '{}': {}", path, err)
        }
    }
}
