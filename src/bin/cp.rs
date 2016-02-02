use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom};
use std::process;

fn main() {
    if let Some(src) = env::args().nth(1) {
        let dsts = env::args().skip(2);

        if dsts.len() < 1 {
            println!("cp: no destination arguments");
            process::exit(2);
        }

        match fs::File::open(src) {
            Ok(mut src_file) => {
                for dst in dsts {
                    match fs::File::create(dst) {
                        Ok(mut dst_file) => {
                            src_file.seek(SeekFrom::Start(0)).unwrap();
                            io::copy(&mut src_file, &mut dst_file).unwrap();
                        },
                        Err(err) => println!("cp: cannot create destination '{}': {}", dst, err)
                    }
                }
            },
            Err(err) => println!("cp: cannot open source '{}': {}", src, err)
        }
    } else {
        println!("cp: no source argument");
        process::exit(1);
    }
}
