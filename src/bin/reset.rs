use std::io::{stdout, Write};

const MAN_PAGE: &'static str = /* @MANSTART{reset} */ r#"
NAME
    reset - terminal initialization

SYNOPSIS
    reset

DESCRIPTION
    Initialize the terminal, clearing the screen and setting all parameters to their default values
"#; /* @MANEND */

fn main() {
    let _ = stdout().write(b"\x1Bc");
}
