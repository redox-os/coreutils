extern crate coreutils;

use std::io::{self, Write};
use coreutils::extra::OptionalExt;

const MAN_PAGE: &'static str = r#"NAME
    basename - strip directory and suffix from filenames

SYNOPSIS
    basename [NAME [SUFFIX]] [OPTION... NAME...]

DESCRIPTION
    Print NAME with any leading directory components removed. If a SUFFIX is provided, the suffix will be removed from NAME.

OPTIONS
    -a
    --multiple
        Support multiple arguments and treat each as a NAME
    -s
    --suffix=SUFFIX
        remove a trailing SUFFIX; implies -a
    -z
    --zero
        end each output line with NUL, not newline
    -h
    --help
        display this help and exit

EXAMPLE
    basename dir/filename.ext
        > filename.ext
    basename dir/filename.ext .ext
        > filename
    basename -a -s .ext one.ext two.ext three.ext
        > one two three

AUTHOR
    Written by Michael Murphy.
"#;

const HELP_INFO:       &'static str = "Try ‘basename --help’ for more information.\n";
const MISSING_OPERAND: &'static str = "missing operand\n";
const REQUIRES_OPTION: &'static str = "option requires an argument -- ‘s’\n";

fn main() {
    let stdout          = io::stdout();
    let mut stdout      = stdout.lock();
    let mut stderr      = io::stderr();
    let mut arguments   = std::env::args().skip(1);
    let mut multiple    = false;
    let mut zero        = false;
    let mut suffix      = String::new();
    let first_name: Option<String>;

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
                            stderr.write_all(REQUIRES_OPTION.as_bytes()).try(&mut stderr);
                            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                            stderr.flush().try(&mut stderr);
                            stdout.flush().try(&mut stderr);
                            std::process::exit(1);
                        }
                    },
                    "-z" | "--zero"     => zero = true,
                    "-h" | "--help"     => {
                        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                        std::process::exit(1);
                    },
                    _ => {
                        stderr.write_all("invalid option -- ‘".as_bytes()).try(&mut stderr);
                        stderr.write_all(argument.as_bytes()).try(&mut stderr);
                        stderr.write_all("’\n".as_bytes()).try(&mut stderr);
                        stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                        stderr.flush().try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                        std::process::exit(1);
                    }
                }
            } else {
                first_name = Some(argument); // Store the last argument so that it isn't lost.
                break
            }
        } else {
            stdout.write_all(MISSING_OPERAND.as_bytes()).try(&mut stderr);
            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
            return;
        }
    }

    // Begin printing the basename of each following argument.
    if let Some(name) = first_name {
        if !multiple {
            // If multiple isn't set and there is more than one
            // argument, the second argument is the suffix.
            if let Some(potential_suffix) = arguments.next() {
                // Check if there is an extra operand and exit if true.
                if let Some(extra_operand) = arguments.next() {
                    stderr.write_all("extra operand ‘".as_bytes()).try(&mut stderr);
                    stderr.write_all(extra_operand.as_bytes()).try(&mut stderr);
                    stderr.write_all("’\n".as_bytes()).try(&mut stderr);
                    stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                    stderr.flush().try(&mut stderr);
                    stdout.flush().try(&mut stderr);
                    std::process::exit(1);
                }

                // Otherwise, set the suffix variable to the argument we found.
                suffix = potential_suffix;
            }
        }
        basename(zero, &name, &suffix, &mut stdout, &mut stderr);
    }

    // If the multiple arguments flag was set, execute a loop to print each name.
    if multiple {
        for argument in arguments {
            basename(zero, &argument, &suffix, &mut stdout, &mut stderr);
        }
    }
}

/// Takes a file path as input and returns the basename of that path. If `zero` is set to true,
/// the name will be printed without a newline. If a suffix is set and the path contains that
/// suffix, the suffix will be removed.
fn basename(zero: bool, input: &str, suffix: &str, stdout: &mut io::StdoutLock, stderr: &mut io::Stderr) {
    // If the suffix variable is set, remove the suffix from the path string.
    let path = if !suffix.is_empty() {
        let (prefix, input_suffix) = input.split_at(input.len() - suffix.len());
        if input_suffix == suffix { prefix } else { input }
    } else {
        input
    };

    // Only print the basename of the the path.
    let new_path = std::path::Path::new(&path);
    match new_path.file_name() {
        Some(base) => {
            if let Some(filename) = base.to_str() {
                stdout.write_all(filename.as_bytes()).try(stderr);
            } else {
                stderr.write_all("invalid unicode in path ‘".as_bytes()).try(stderr);
                stderr.write_all(path.as_bytes()).try(stderr);
                stderr.write_all("’\n".as_bytes()).try(stderr);
                stderr.flush().try(stderr);
                std::process::exit(1);
            }

        },
        None => {
            stderr.write_all("invalid path ‘".as_bytes()).try(stderr);
            stderr.write_all(path.as_bytes()).try(stderr);
            stderr.write_all("’\n".as_bytes()).try(stderr);
            stderr.flush().try(stderr);
            std::process::exit(1);
        }
    };

    // If zero is not enabled, print a newline.
    if !zero { stdout.write_all(b"\n").try(stderr); }
}
