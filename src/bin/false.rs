//#![deny(warnings)]

use std::process;

const MAN_PAGE: &'static str = /* @MANSTART{false} */ r#"
NAME
    false - do nothing, unsuccessfully

SYNOPSIS
    false

DESCRIPTION
    Exit with a status code indicating failure.
"#; /* @MANEND */

fn main() {
    process::exit(1);
}
