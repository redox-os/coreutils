#![deny(warnings)]
#![feature(fs_time)]
extern crate coreutils;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

use coreutils::extra::OptionalExt;

const MAN_PAGE: &'static str = r#"NAME
    test - perform tests on files and text

SYNOPSIS
    test [EXPRESSION]

DESCRIPTION
    Tests the expressions given and returns an exit status of 0 if true, else 1.

OPTIONS
    -n STRING
        the length of STRING is nonzero

    STRING
        equivalent to -n STRING

    -z STRING
        the length of STRING is zero

    STRING = STRING
        the strings are equivalent

    STRING != STRING
        the strings are not equal

    INTEGER -eq INTEGER
        the integers are equal

    INTEGER -ge INTEGER
        the first INTEGER is greater than or equal to the first INTEGER

    INTEGER -gt INTEGER
        the first INTEGER is greater than the first INTEGER

    INTEGER -le INTEGER
        the first INTEGER is less than or equal to the first INTEGER

    INTEGER -lt INTEGER
        the first INTEGER is less than the first INTEGER

    INTEGER -ne INTEGER
        the first INTEGER is not equal to the first INTEGER

    FILE -ef FILE
        both files have the same device and inode numbers

    FILE -nt FILE
        the first FILE is newer than the second FILE

    FILE -ot FILE
        the first file is older than the second FILE

    -b FILE
        FILE exists and is a block device

    -c FILE
        FILE exists and is a character device

    -d FILE
        FILE exists and is a directory

    -e FILE
        FILE exists

    -f FILE
        FILE exists and is a regular file

    -h FILE
        FILE exists and is a symbolic link (same as -L)

    -L FILE
        FILE exists and is a symbolic link (same as -h)

    -r FILE
        FILE exists and read permission is granted

    -s FILE
        FILE exists and has a file size greater than zero

    -w FILE
        FILE exists and write permission is granted

    -x FILE
        FILE exists and execute (or search) permission is granted

EXAMPLES
    Test if the file exists:
        test -e FILE && echo "The FILE exists" || echo "The FILE does not exist"

    Test if the file exists and is a regular file, and if so, write to it:
        test -f FILE && echo "Hello, FILE" >> FILE || echo "Cannot write to a directory"

    Test if 10 is greater than 5:
        test 10 -gt 5 && echo "10 is greater than 5" || echo "10 is not greater than 5"

    Test if the user is running a 64-bit OS (POSIX environment only):
        test $(getconf LONG_BIT) = 64 && echo "64-bit OS" || echo "32-bit OS"

AUTHOR
    Written by Michael Murphy.
"#;

const SUCCESS: i32 = 0;
const FAILED:  i32 = 1;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut arguments = std::env::args().skip(1);
    // TODO: Implement support for evaluating multiple expressions
    if let Some(first_argument) = arguments.next() {
        if first_argument.as_str() == "--help" {
            stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(SUCCESS);
        }
        let mut characters = first_argument.chars().take(2);
        match characters.next().unwrap() {
            '-' => match_flag_argument(characters.next(), arguments.next()),
            _   => evaluate_expression(first_argument.as_str(), arguments.next(), arguments.next(),
                                       &mut stderr),
        }
    } else {
        exit(FAILED);
    }
}

/// Evaluate an expression of `VALUE -OPERATOR VALUE`.
fn evaluate_expression(first: &str, operator: Option<String>, second_argument: Option<String>,
                       stderr: &mut io::Stderr) {
    match operator {
        Some(op) => {
            let op = op.as_str();
            match second_argument {
                Some(second) => {
                    let second = second.as_str();
                    match op {
                        "=" | "==" => evaluate_bool(first == second),
                        "!=" => evaluate_bool(first != second),
                        "-eq" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left == right);
                        },
                        "-ge" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left >= right);
                        },
                        "-gt" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left > right);
                        },
                        "-le" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left <= right);
                        },
                        "-lt" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left < right);
                        },
                        "-ne" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left != right)
                        },
                        "-ef" => files_have_same_device_and_inode_numbers(first, second),
                        "-nt" => file_is_newer_than(first, second),
                        "-ot" => file_is_older_than(first, second),
                        _          => {
                            stderr.write_all(b"unknown condition: ").try(stderr);
                            stderr.write_all(op.as_bytes()).try(stderr);
                            stderr.write_all(&[b'\n']).try(stderr);
                            stderr.flush().try(stderr);
                            exit(FAILED);
                        }
                    }
                },
                None => {
                    stderr.write_all(b"parse error: condition expected\n").try(stderr);
                    stderr.flush().try(stderr);
                    exit(FAILED);
                }
            }
        },
        None => string_is_nonzero(Some(String::from(first)))
    }
}

/// Exits SUCCESS if both files have the same device and inode numbers
fn files_have_same_device_and_inode_numbers(first: &str, second: &str) {
    evaluate_bool(get_dev_and_inode(first) == get_dev_and_inode(second));
}

/// Obtains the device and inode numbers of the file specified
fn get_dev_and_inode(filename: &str) -> (u64, u64) {
    match fs::metadata(filename) {
        Ok(file) => (file.dev(), file.ino()),
        Err(_)   => exit(FAILED)
    }
}

/// Exits SUCCESS if the first file is newer than the second file.
fn file_is_newer_than(first: &str, second: &str) {
    evaluate_bool(get_modified_file_time(first) < get_modified_file_time(second));
}

/// Exits SUCCESS if the first file is older than the second file.
fn file_is_older_than(first: &str, second: &str) {
    evaluate_bool(get_modified_file_time(first) < get_modified_file_time(second));
}

/// Obtain the time the file was last modified as a `SystemTime` type.
fn get_modified_file_time(filename: &str) -> std::time::SystemTime {
    match fs::metadata(filename) {
        Ok(file) => match file.modified() {
            Ok(time) => return time,
            Err(_)   => exit(FAILED)
        },
        Err(_) => exit(FAILED)
    }
}

/// Attempt to parse a &str as a usize.
fn parse_integers(left: &str, right: &str, stderr: &mut io::Stderr) -> (usize, usize) {
    let mut parse_integer = |input: &str| -> usize {
        if let Ok(integer) = input.parse::<usize>() {
            integer
        } else {
            stderr.write_all(b"integer expression expected: ").try(stderr);
            stderr.write_all(input.as_bytes()).try(stderr);
            stderr.write_all(&[b'\n']).try(stderr);
            stderr.flush().try(stderr);
            exit(FAILED);
        }
    };
    (parse_integer(left), parse_integer(right))
}

/// Matches flag arguments to their respective functionaity when the `-` character is detected.
fn match_flag_argument(character: Option<char>, arguments: Option<String>) {
    if let Some(second_character) = character {
        // TODO: Implement missing flags
        match second_character {
            'b' => file_is_block_device(arguments),
            'c' => file_is_character_device(arguments),
            'd' => file_is_directory(arguments),
            'e' => file_exists(arguments),
            'f' => file_is_regular(arguments),
            //'g' => file_is_set_group_id(arguments),
            //'G' => file_is_owned_by_effective_group_id(arguments),
            'h' => file_is_symlink(arguments),
            //'k' => file_has_sticky_bit(arguments),
            'L' => file_is_symlink(arguments),
            //'O' => file_is_owned_by_effective_user_id(arguments),
            //'p' => file_is_named_pipe(arguments),
            'r' => file_has_read_permission(arguments),
            's' => file_size_is_greater_than_zero(arguments),
            'S' => file_is_socket(arguments),
            //'t' => file_descriptor_is_opened_on_a_terminal(arguments),
            'w' => file_has_write_permission(arguments),
            'x' => file_has_execute_permission(arguments),
            'n' => string_is_nonzero(arguments),
            'z' => string_is_zero(arguments),
            _ => exit(SUCCESS),
        }
    } else {
        exit(SUCCESS);
    }
}

/// Exits SUCCESS if the file size is greather than zero.
fn file_size_is_greater_than_zero(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.len() > 0),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file has read permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective read bits.
fn file_has_read_permission(file: Option<String>) {
    const USER_BIT:  u32 = 0b100000000;
    const GROUP_BIT: u32 = 0b100000;
    const GUEST_BIT: u32 = 0b100;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { exit(SUCCESS); }
                if permissions & GROUP_BIT == GROUP_BIT { exit(SUCCESS); }
                if permissions & GUEST_BIT == GUEST_BIT { exit(SUCCESS); }
                exit(FAILED);
            },
            Err(_) => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file has write permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective write bits.
fn file_has_write_permission(file: Option<String>) {
    const USER_BIT:  u32 = 0b10000000;
    const GROUP_BIT: u32 = 0b10000;
    const GUEST_BIT: u32 = 0b10;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { exit(SUCCESS); }
                if permissions & GROUP_BIT == GROUP_BIT { exit(SUCCESS); }
                if permissions & GUEST_BIT == GUEST_BIT { exit(SUCCESS); }
                exit(FAILED);
            },
            Err(_) => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file has execute permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective execute bits.
fn file_has_execute_permission(file: Option<String>) {
    const USER_BIT:  u32 = 0b1000000;
    const GROUP_BIT: u32 = 0b1000;
    const GUEST_BIT: u32 = 0b1;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { exit(SUCCESS); }
                if permissions & GROUP_BIT == GROUP_BIT { exit(SUCCESS); }
                if permissions & GUEST_BIT == GUEST_BIT { exit(SUCCESS); }
                exit(FAILED);
            },
            Err(_) => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file argument is a socket
fn file_is_socket(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_socket()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file argument is a block device
fn file_is_block_device(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_block_device()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file argument is a character device
fn file_is_character_device(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_char_device()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file exists
fn file_exists(file: Option<String>) {
    match file {
        Some(filepath) => evaluate_bool(Path::new(&filepath).exists()),
        None           => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file is a regular file
fn file_is_regular(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_file()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file is a directory
fn file_is_directory(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_dir()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the file is a symbolic link
fn file_is_symlink(file: Option<String>) {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_symlink()),
            Err(_)       => exit(FAILED)
        },
        None => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the string is not empty
fn string_is_nonzero(string: Option<String>) {
    match string {
        Some(string) => evaluate_bool(!string.is_empty()),
        None         => exit(SUCCESS)
    }
}

/// Exits SUCCESS if the string is empty
fn string_is_zero(string: Option<String>) {
    match string {
        Some(string) => evaluate_bool(string.is_empty()),
        None         => exit(SUCCESS)
    }
}

// Convert a boolean to it's respective exit code.
fn evaluate_bool(input: bool) { if input { exit(SUCCESS); } else { exit(FAILED); } }
