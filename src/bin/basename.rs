extern crate coreutils;

use std::io::{self,Write};
use coreutils::extra::{OptionalExt};

static MAN_PAGE: &'static str = r#"
NAME
    basename - strip directory and suffix from filenames
SYNOPSIS
    basename NAME [SUFFIX]
    basename OPTION... NAME...
DESCRIPTION
    Print NAME with any leading directory components removed. If specified, also remove a trailing SUFFIX.
OPTIONS
    -a, --multiple
        support multiple arguments and treat each as a NAME
    -s, --suffix=SUFFIX
        remove a trailing SUFFIX; implies -a
    -z, --zero
        end each output line with NUL, not newline
    -h, --help
        display this help and exit
AUTHOR
    Written by Michael Murphy.
"#;

static HELP_INFO: &'static str       = "Try 'basename --help' for more information.";
static MISSING_OPERAND: &'static str = "basename: missing operand";
static REQUIRES_OPTION: &'static str = "basename: option requires an argument -- 's'";

fn main() {
    let stdout          = io::stdout();
    let mut stdout      = stdout.lock();
    let mut stderr      = io::stderr();
    let mut arguments   = std::env::args().skip(1);
    let mut multiple    = false;
    let mut zero        = false;
    let mut suffix      = String::new();
    let mut first_name  = String::new();

    // Check input arguments for options.
    loop {
        if let Some(argument) = arguments.next() {
            // If the first character begins with a `-` then it is an option.
            if argument.chars().take(1).next().unwrap_or(' ') == '-' {
                match argument.as_str() {
                    "-a" | "--multiple" => multiple = true,
                    "-s" | "--suffix"   => {
                        if let Some(arg) = arguments.next() {
                            suffix = arg;
                            multiple = true;
                        } else {
                            stdout.write_all(REQUIRES_OPTION.as_bytes()).try(&mut stderr);
                            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                            return;
                        }
                    },
                    "-z" | "--zero"     => zero = true,
                    "-h" | "--help"     => {
                        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                        return;
                    },
                    _ => {
                        println!("basename: invalid option -- '{}'", argument);
                        stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                        return;
                    }
                }
            } else {
                first_name = argument; // Store the last argument so that it isn't lost.
                break
            }
        } else {
            stdout.write_all(MISSING_OPERAND.as_bytes()).try(&mut stderr);
            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
            return;
        }
    }

    // Begin printing the basename of each following argument.
    basename(zero, &first_name, &suffix);
    if multiple {
        for argument in arguments {
            basename(zero, &argument, &suffix);
        }
    }
}

/// Takes a file path as input and returns the basename of that path. If `zero` is set to true,
/// the name will be printed without a newline. If a suffix is set and the path contains that
/// suffix, the suffix will be removed.
fn basename(zero: bool, path: &str, suffix: &str) {
    let mut path = String::from(path);
    let path_len = path.len();

    // If the suffix variable is set, remove the suffix.
    if path.ends_with(suffix) {
        path.truncate(path_len - suffix.len());
    }

    // Take only the basename of the path
    let new_path = std::path::Path::new(&path);
    let mut output = match new_path.file_name() {
        Some(base)     => String::from(base.to_str().unwrap()),
        None           => unreachable!()
    };

    // If zero is enabled, do not print a newline.
    if !zero { output.push('\n'); }

    // Print the output directly to stdout.
    io::stdout().write_all(output.as_bytes()).try(&mut io::stderr())
}
