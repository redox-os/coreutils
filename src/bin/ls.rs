use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).unwrap_or(".".to_string());

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
