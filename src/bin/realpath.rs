use std::env;
use std::fs::File;
use std::process;

fn main() {
    if env::args().count() < 2 {
        println!("realpath: no arguments");
        process::exit(1);
    }

    for path in env::args().skip(1) {
        match File::open(path) {
            Ok(mut file) => {
                match file.path() {
                    Ok(realpath) => println!("{}", realpath.display()),
                    Err(err) => println!("realpath: cannot get path of '{}': {}", path, err)
                }
            },
            Err(err) => println!("realpath: cannot open '{}': {}", path, err)
        }
    }
}
