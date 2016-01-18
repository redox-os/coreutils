#[macro_use]
extern crate clap;

use clap::{App, Arg};

use std::fs;

fn main() {
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

    if fs::remove_file(path).is_err() {
        println!("Failed to remove: {}", path);
    }
}
