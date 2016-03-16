use std::io::{stdout, Write};

fn main() {
    let _ = stdout().write(b"\x1Bc");
}
