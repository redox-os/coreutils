//#[macro_use]
//extern crate clap;

//use clap::{App, Arg};

use std::env;
use std::fs;
use std::process;

fn main() {
    /*
    let matches = App::new("rm")
                      .version("0.0.1")
                      .author("Redox Developers")
                      .about("Delete files and directories")
                      .arg(Arg::with_name("item")
                               .help("Item to remove")
                               .required(true)
                               .multiple(true))
                      .arg(Arg::with_name("interactive")
                               .help("Prompt before each removal")
                               .short("i")
                               .long("interactive"))
                      .arg(Arg::with_name("force")
                               .help("Ignore any errors or prompts. Guaranteed to delete the \
                                      file(s) or directory(s).")
                               .short("f")
                               .long("force"))
                      .arg(Arg::with_name("verbose")
                               .help("Print extra info")
                               .short("v")
                               .long("verbose"))
                      .get_matches();

    // TODO support arguments

    let path = matches.value_of("item").unwrap();
    */

    if env::args().count() < 2 {
        println!("rm: no arguments");
        process::exit(1);
    }

    for path in env::args().skip(1) {
        if let Err(err) = fs::remove_file(&path) {
            println!("rm: cannot remove '{}': {}", path, err);
        }
    }
}
