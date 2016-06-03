#![deny(warnings)]

use std::process;

const MAN_PAGE: &'static str = /* @MANSTART{true} */ r#"
NAME
    true - do nothing, successfully

SYNOPSIS
    true

DESCRIPTION
    Exit with a status code indicating success.
"#; /* @MANEND */

fn main() {
    process::exit(0);
}
