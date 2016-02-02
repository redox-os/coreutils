use std::env;

fn main() {
    let mut newline = true;
    for arg in env::args().skip(1) {
        if *arg == "-n" {
            newline = false;
        } else {
            print!("{} ", arg);
        }
    }
    if newline {
        print!("\n");
    }
}
