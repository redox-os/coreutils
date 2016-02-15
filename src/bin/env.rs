use std::env;
use std::io::{stdout, Write};

fn main() {
    let mut stdout = stdout();

    for (key, value) in env::vars() {
        writeln!(stdout, "{}={}", key, value);
    }
}
