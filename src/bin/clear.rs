use std::io::{stdout, Write};

fn main() {
    let _ = stdout().write(b"\x1B[2J");
}
