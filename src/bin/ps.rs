use std::fs::File;
use std::io;

fn main() {
    let mut file = File::open("context:").unwrap();
    io::copy(&mut file, &mut io::stdout()).unwrap();
    println!("");
}
