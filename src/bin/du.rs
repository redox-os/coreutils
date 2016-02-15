extern crate coreutils;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{stdout, Write, Seek, SeekFrom};
use coreutils::extra::OptionalExt;

fn print(path: &str) {
    let mut stdout = stdout();
    let mut entries = Vec::new();

    let dir = fs::read_dir(path).try();

    for entry_result in dir {
        match entry_result {
            Ok(entry) => {
                let directory = match entry.file_type() {
                    Ok(file_type) => file_type.is_dir(),
                    Err(err) => {
                        writeln!(stdout, "warning: failed to read file type: {}", err);
                        false
                    }
                };

                if let Some(path_str) = entry.file_name().to_str() {
                    entries.push(path_str.to_owned());
                    if directory {
                        entries.last_mut().unwrap().push('/');
                    }
                } else {
                    writeln!(stdout, "warning: failed to convert path to string");
                }
            }
            Err(err) => {
                writeln!(stdout, "warning: failed to read entry: {}", err).try();
            },
        }
    }

    entries.sort();

    for entry in entries.iter() {
        match File::open(entry) {
            Ok(mut file) => {
                match file.seek(SeekFrom::End(0)) {
                    Ok(size) => {
                        writeln!(stdout, "{}\t{}", (size + 1023) / 1024, entry);
                    },
                    Err(err) => {
                        writeln!(stdout, "warning: cannot seek file '{}': {}", entry, err);
                    }
                }
            },
            Err(err) => {
                println!("warning: cannot read file '{}': {}", entry, err);
            }
        }
    }
}
fn main() {
    if let Some(ref x) = env::args().nth(1) {
        print(x);
    } else {
        print(".");
    }
}
