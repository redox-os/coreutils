#![deny(warnings)]
extern crate extra;

use std::io::{self, Write};
use extra::option::OptionalExt;
use std::process;

const MAN_PAGE: &'static str = /* @MANSTART{basename} */ r#"
NAME
    basename - strip directory and suffix from filenames

SYNOPSIS
    basename [-z --zero] [-s | --suffix] SUFFIX [-a | --multiple] NAME...

DESCRIPTION
    Print NAME with any leading directory components removed. If a SUFFIX is provided, the suffix
    will be removed from NAME.

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
"#; /* @MANEND */

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
    let first_path: String;

    loop {
        if let Some(argument) = arguments.next() {
            // If the first character begins with a `-` then it is an option.
            if argument.chars().take(1).next().unwrap_or(' ') == '-' {
                match argument.as_str() {
                    "-a" | "--multiple" => multiple = true,
                    "-s" | "--suffix" => {
                        if let Some(arg) = arguments.next() {
                            suffix = arg;
                            multiple = true;
                        } else {
                            stderr.write_all(REQUIRES_OPTION.as_bytes()).try(&mut stderr);
                            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                            stderr.flush().try(&mut stderr);
                            process::exit(1);
                        }
                    },
                    "-z" | "--zero" => zero = true,
                    "-h" | "--help" => {
                        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                        process::exit(0);
                    },
                    _ => {
                        stderr.write_all("invalid option -- ‘".as_bytes()).try(&mut stderr);
                        stderr.write_all(argument.as_bytes()).try(&mut stderr);
                        stderr.write_all("’\n".as_bytes()).try(&mut stderr);
                        stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                        stderr.flush().try(&mut stderr);
                        process::exit(1);
                    }
                }
            } else {
                first_path = argument; // Store the last argument so that it isn't lost.
                break
            }
        } else {
            stdout.write_all(MISSING_OPERAND.as_bytes()).try(&mut stderr);
            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
            process::exit(1);
        }
    }

    if multiple {
        // Print the first path's basename
        basename(zero, &first_path, &suffix, &mut stdout, &mut stderr);
        // and then print every pathname after that.
        for path in arguments { basename(zero, &path, &suffix, &mut stdout, &mut stderr); }
    } else {
        // If there is an additional variable, set this variable as the suffix to remove
        arguments.next().map(|potential_suffix| {
            // If there is an extra variable after that, print an error about an extra operand
            arguments.next().map(|extra_operand| {
                stderr.write_all("extra operand ‘".as_bytes()).try(&mut stderr);
                stderr.write_all(extra_operand.as_bytes()).try(&mut stderr);
                stderr.write_all("’\n".as_bytes()).try(&mut stderr);
                stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                stderr.flush().try(&mut stderr);
                process::exit(1);
            });
            suffix = potential_suffix;
        });
        basename(zero, &first_path, &suffix, &mut stdout, &mut stderr);
    }
}

/// Takes a file path as input and returns the basename of that path. If `zero` is set to true,
/// the name will be printed without a newline. If a suffix is set and the path contains that
/// suffix, the suffix will be removed.
fn basename(zero: bool, path: &str, suffix: &str, stdout: &mut io::StdoutLock, stderr: &mut io::Stderr) {
    match get_basename(path, suffix) {
        Ok(filename) => stdout.write_all(filename.as_bytes()).try(stderr),
        Err(why) => {
            // An error occurred, so this will select the correct message to print
            let err_message = match why {
                BaseError::Path    => "invalid path ‘".as_bytes(),
                BaseError::Unicode => "invalid unicode in path ‘".as_bytes()
            };
            stderr.write_all(err_message).try(stderr);
            stderr.write_all(path.as_bytes()).try(stderr);
            stderr.write_all("’\n".as_bytes()).try(stderr);
            stderr.flush().try(stderr);
            process::exit(1);
        }
    }

    if !zero { stdout.write_all(b"\n").try(stderr); }
}

#[derive(Debug, PartialEq)]
enum BaseError { Path, Unicode }

/// Either return the basename as a byte slice or return a `BaseError`
fn get_basename<'a>(path: &'a str, suffix: &str) -> Result<&'a str, BaseError> {
    // Remove the suffix from the path and attempt to collect the file name in that path
    std::path::Path::new(remove_suffix(path, suffix)).file_name()
        // If there was an error in obtaining the filename from the path, return an error
        .map_or(Err(BaseError::Path), |base| {
            // Convert the filename back into a string, else return a unicode error.
            base.to_str().map_or(Err(BaseError::Unicode), |filename| Ok(filename))
        })
}

/// Removes the suffix from the input
fn remove_suffix<'a>(input: &'a str, suffix: &str) -> &'a str {
    if suffix.is_empty() {
        input
    } else {
        let (prefix, input_suffix) = input.split_at(input.chars().count() - suffix.chars().count());
        if input_suffix == suffix { prefix } else { input }
    }
}

#[test]
fn test_basename() {
    assert_eq!(Ok("a"), get_basename("a.b", ".b"));
}
