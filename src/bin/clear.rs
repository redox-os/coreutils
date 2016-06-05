//#![deny(warnings)]

use std::io::{stdout, Write};

const MAN_PAGE: &'static str = /* @MANSTART{clear} */ r#"
NAME
    clear - clear the terminal screen

SYNOPSIS
    clear

DESCRIPTION
    Clear the screen, if possible
"#; /* @MANEND */

fn main() {
    let _ = stdout().write(b"\x1B[2J");
}
