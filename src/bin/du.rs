use std::env;
use std::fs;
use std::fs::File;
use std::io::{Seek, SeekFrom};

fn main() {
    let path = env::args().nth(1).map_or(".", |p| *p);

    let mut entries = Vec::new();

    match fs::read_dir(&path) {
        Ok(dir) => {
            for entry_result in dir {
                match entry_result {
                    Ok(entry) => {
                        let directory = match entry.file_type() {
                            Ok(file_type) => file_type.is_dir(),
                            Err(err) => {
                                println!("du: failed to read file type: {}", err);
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
                            None => println!("du: failed to convert path to string"),
                        }
                    }
                    Err(err) => println!("du: failed to read entry: {}", err),
                }
            }
        }
        Err(err) => println!("du: failed to open directory '{}': {}", path, err),
    }

    entries.sort();

    for entry in entries.iter() {
        match File::open(entry) {
            Ok(mut file) => {
                match file.seek(SeekFrom::End(0)) {
                    Ok(size) => {
                        println!("{}\t{}", (size + 1023)/1024, entry);
                    },
                    Err(err) => {
                        println!("du: cannot seek file '{}': {}", entry, err);
                    }
                }
            },
            Err(err) => {
                println!("du: cannot read file '{}': {}", entry, err);
            }
        }
    }
}
