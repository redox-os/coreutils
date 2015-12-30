use std::env;
use std::fs;
use std::io;

fn main() {
    let paths = env::args().skip(1);

    if paths.len() == 0 {
        io::copy(&mut io::stdin(), &mut io::stdout());
    } else {
        for path in paths {
            let mut file: Box<io::Read> = if path == "-" {
                Box::new(io::stdin())
            } else {
                match fs::File::open(&path).map(Box::new) {
                    Ok(v) => v,
                    Err(err) => panic!("Cannot open file {}: {}", path, err)
                }
            };

            io::copy(&mut *file, &mut io::stdout());
        }

    }
}
