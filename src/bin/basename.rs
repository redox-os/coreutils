#![deny(warnings)]

extern crate extra;
extern crate arg_parser;

use std::io::{self, Write};
use std::env;
use std::process;
use arg_parser::ArgParser;
use extra::option::OptionalExt;

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
    let mut parser = ArgParser::new(4)
        .add_opt("s", "suffix")
        .add_flag(&["a", "multiple"])
        .add_flag(&["z", "zero"])
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        process::exit(0);
    }
    if parser.found("suffix") {
        if parser.get_opt(&'s').is_none() && parser.get_opt("suffix").is_none() {
            stderr.write_all(REQUIRES_OPTION.as_bytes()).try(&mut stderr);
            stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
            stderr.flush().try(&mut stderr);
            process::exit(1);
        }
        *parser.flag("multiple") = true;
    }
    if parser.args.is_empty() {
        stdout.write_all(MISSING_OPERAND.as_bytes()).try(&mut stderr);
        stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
        process::exit(1);
    }
    if let Err(err) = parser.found_invalid() {
        stderr.write_all(err.as_bytes()).try(&mut stderr);
        stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        process::exit(1);
    }

    if parser.found("multiple") {
        for path in &parser.args {
            basename(&path, &parser, &mut stdout, &mut stderr);
        }
    } else {
        // If there is an additional variable, set this variable as the suffix to remove
        if let Some(potential_suffix) = parser.args.get(1).map(|s| (*s).clone()) {
            *parser.opt(&'s') = potential_suffix.clone();
            *parser.opt("suffix") = potential_suffix;
            // If there is an extra variable after that, print an error about an extra operand
            if let Some(extra_operand) = parser.args.get(2) {
                stderr.write_all("extra operand ‘".as_bytes()).try(&mut stderr);
                stderr.write_all(extra_operand.as_bytes()).try(&mut stderr);
                stderr.write_all("’\n".as_bytes()).try(&mut stderr);
                stdout.write_all(HELP_INFO.as_bytes()).try(&mut stderr);
                stderr.flush().try(&mut stderr);
                process::exit(1);
            }
        }
        basename(&parser.args[0], &parser, &mut stdout, &mut stderr);
    }
}

/// Takes a file path as input and returns the basename of that path. If `zero` is set to true,
/// the name will be printed without a newline. If a suffix is set and the path contains that
/// suffix, the suffix will be removed.
fn basename(path: &str, parser: &ArgParser, stdout: &mut io::StdoutLock, stderr: &mut io::Stderr) {
    match get_basename(path, &parser.get_opt("suffix").fail("No arguments. Use --help to see the usage.", stderr)) {
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

    if parser.found("zero") {
        stdout.write_all(b"\n").try(stderr);
    }
}

#[derive(Debug, PartialEq)]
enum BaseError { Path, Unicode }

/// Either return the basename as a byte slice or return a `BaseError`
fn get_basename<'a>(input: &'a str, suffix: &str) -> Result<&'a str, BaseError> {
    // Remove the suffix from the path and attempt to collect the file name in that path
    std::path::Path::new({
            let (prefix, input_suffix) = input.split_at(input.len() - suffix.len());
            if input_suffix == suffix { prefix } else { input }
    })
        .file_name()
        // If there was an error in obtaining the filename from the path, return an error
        .map_or(Err(BaseError::Path), |base| {
            // Convert the filename back into a string, else return a unicode error.
            base.to_str().map_or(Err(BaseError::Unicode), |filename| Ok(filename))
        })
}

#[test]
fn test_basename() {
    assert_eq!(Ok("a"), get_basename("a.b", ".b"));
}
