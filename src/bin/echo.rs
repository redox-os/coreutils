use std::env;
use std::io::{stdout, Write};

fn main() {
    let mut stdout = stdout();

    let mut newline = true;
    for arg in env::args().skip(1) {
        if arg == "-n" {
            newline = false;
        } else {
            write!(stdout, "{} ", arg);
        }
    }
    if newline {
        write!(stdout, "\n");
    }
}
