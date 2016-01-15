#[macro_use]
extern crate clap;

use clap::{App, Arg};

use std::fs;

fn main() {
    let matches = App::new("ls")
                      .version("0.0.1")
                      .author("Redox Developers")
                      .about("List files and directories in the specified path (default = \
                              current)")
                      .arg(Arg::with_name("dir")
                               .help("Directory to list")
                               .index(1))
                      .arg(Arg::with_name("long")
                               .short("l")
                               .long("long")
                               .help("Use long display format"))
                      .arg(Arg::with_name("onecol")
                               .short("1")
                               .help("Display files in one column"))
                      .arg(Arg::with_name("all")
                               .short("a")
                               .long("all")
                               .help("Show files whose names begin with a '.'"))
                      .get_matches();

    let path = matches.value_of("dir").unwrap_or(".");

    let mut entries = Vec::new();

    match fs::read_dir(&path) {
        Ok(dir) => {
            for entry_result in dir {
                match entry_result {
                    Ok(entry) => {
                        let directory = match entry.file_type() {
                            Ok(file_type) => file_type.is_dir(),
                            Err(err) => {
                                println!("Failed to read file type: {}", err);
                                false
                            }
                        };

                        match entry.file_name().to_str() {
                            Some(path_str) => {
                                if directory {
                                    entries.push(path_str.to_string() + "/")
                                } else {
                                    entries.push(path_str.to_string())
                                }
                            }
                            None => println!("Failed to convert path to string"),
                        }
                    }
                    Err(err) => println!("Failed to read entry: {}", err),
                }
            }
        }
        Err(err) => println!("Failed to open directory: {}: {}", path, err),
    }

    entries.sort();

    for entry in entries {
        println!("{}", entry);
    }
}
