use std::env;
use std::fs;
use std::io;

fn main() {
    // TODO support clap

    let paths = env::args().skip(1);

    if paths.len() == 0 {
        io::copy(&mut io::stdin(), &mut io::stdout()).unwrap();
    } else {
        for path in paths {
            if path == "-" {
                io::copy(&mut io::stdin(), &mut io::stdout()).unwrap();
            } else {
                match fs::File::open(&path).map(Box::new) {
                    Ok(mut file) => {
                        io::copy(&mut *file, &mut io::stdout()).unwrap();
                    },
                    Err(err) => println!("cat: cannot open file {}: {}", path, err)
                }
            };
        }
    }
}
