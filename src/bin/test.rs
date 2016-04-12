#![deny(warnings)]
#![feature(fs_time)]
extern crate extra;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{test} */ r#"
NAME
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

    -S FILE
        FILE exists and is a socket

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
"#; /* @MANEND */

const SUCCESS: i32 = 0;
const FAILED:  i32 = 1;

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    // TODO: Implement support for evaluating multiple expressions
    let expression = std::env::args().skip(1).collect::<Vec<String>>();
    exit(evaluate_arguments(expression, &mut stdout, &mut stderr));
}

fn evaluate_arguments(arguments: Vec<String>, stdout: &mut std::io::StdoutLock, stderr: &mut std::io::Stderr) -> i32 {
    if let Some(arg) = arguments.first() {
        if arg.as_str() == "--help" {
            stdout.write_all(MAN_PAGE.as_bytes()).try(stderr);
            stdout.flush().try(stderr);
            return SUCCESS;
        }
        let mut characters = arg.chars().take(2);
        return match characters.next().unwrap() {
            '-' => match_flag_argument(characters.next(), arguments.get(1)),
            _   => evaluate_expression(arg.as_str(), arguments.get(1), arguments.get(2), stderr),
        };
    } else {
        FAILED
    }
}

/// Evaluate an expression of `VALUE -OPERATOR VALUE`.
fn evaluate_expression(first: &str, operator: Option<&String>, second_argument: Option<&String>,
                       stderr: &mut io::Stderr) -> i32 {
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
                            evaluate_bool(left == right)
                        },
                        "-ge" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left >= right)
                        },
                        "-gt" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left > right)
                        },
                        "-le" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left <= right)
                        },
                        "-lt" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left < right)
                        },
                        "-ne" => {
                            let (left, right) = parse_integers(first, second, stderr);
                            evaluate_bool(left != right)
                        },
                        "-ef" => files_have_same_device_and_inode_numbers(first, second),
                        "-nt" => file_is_newer_than(first, second),
                        "-ot" => file_is_newer_than(second, first),
                        _          => {
                            stderr.write_all(b"unknown condition: ").try(stderr);
                            stderr.write_all(op.as_bytes()).try(stderr);
                            stderr.write_all(&[b'\n']).try(stderr);
                            stderr.flush().try(stderr);
                            FAILED
                        }
                    }
                },
                None => {
                    stderr.write_all(b"parse error: condition expected\n").try(stderr);
                    stderr.flush().try(stderr);
                    FAILED
                }
            }
        },
        None => string_is_nonzero(Some(&String::from(first)))
    }
}

/// Exits SUCCESS if both files have the same device and inode numbers
fn files_have_same_device_and_inode_numbers(first: &str, second: &str) -> i32 {
    let left = match get_dev_and_inode(first) {
        Some(values) => values,
        None         => return FAILED
    };

    let right = match get_dev_and_inode(second) {
        Some(values) => values,
        None         => return FAILED
    };

    evaluate_bool(left == right)
}

/// Obtains the device and inode numbers of the file specified
fn get_dev_and_inode(filename: &str) -> Option<(u64, u64)> {
    match fs::metadata(filename) {
        Ok(file) => Some((file.dev(), file.ino())),
        Err(_)   => None
    }
}

/// Exits SUCCESS if the first file is newer than the second file.
fn file_is_newer_than(first: &str, second: &str) -> i32 {
    let left = match get_modified_file_time(first) {
        Some(time) => time,
        None       => return FAILED
    };

    let right = match get_modified_file_time(second) {
        Some(time) => time,
        None       => return FAILED
    };
    evaluate_bool(left > right)
}

/// Obtain the time the file was last modified as a `SystemTime` type.
fn get_modified_file_time(filename: &str) -> Option<std::time::SystemTime> {
    match fs::metadata(filename) {
        Ok(file) => match file.modified() {
            Ok(time) => Some(time),
            Err(_)   => None
        },
        Err(_) => None
    }
}

/// Attempt to parse a &str as a usize.
fn parse_integers(left: &str, right: &str, stderr: &mut io::Stderr) -> (Option<usize>, Option<usize>) {
    let mut parse_integer = |input: &str| -> Option<usize> {
        if let Ok(integer) = input.parse::<usize>() {
            Some(integer)
        } else {
            stderr.write_all(b"integer expression expected: ").try(stderr);
            stderr.write_all(input.as_bytes()).try(stderr);
            stderr.write_all(&[b'\n']).try(stderr);
            stderr.flush().try(stderr);
            None
        }
    };
    (parse_integer(left), parse_integer(right))
}

/// Matches flag arguments to their respective functionaity when the `-` character is detected.
fn match_flag_argument(character: Option<char>, arguments: Option<&String>) -> i32 {
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
            'h' | 'L' => file_is_symlink(arguments),
            //'k' => file_has_sticky_bit(arguments),
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
            _ => SUCCESS,
        }
    } else {
        SUCCESS
    }
}

/// Exits SUCCESS if the file size is greather than zero.
fn file_size_is_greater_than_zero(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.len() > 0),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file has read permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective read bits.
fn file_has_read_permission(file: Option<&String>) -> i32 {
    const USER_BIT:  u32 = 0b100000000;
    const GROUP_BIT: u32 = 0b100000;
    const GUEST_BIT: u32 = 0b100;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { return SUCCESS; }
                if permissions & GROUP_BIT == GROUP_BIT { return SUCCESS; }
                if permissions & GUEST_BIT == GUEST_BIT { return SUCCESS; }
                FAILED
            },
            Err(_) => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file has write permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective write bits.
fn file_has_write_permission(file: Option<&String>) -> i32 {
    const USER_BIT:  u32 = 0b10000000;
    const GROUP_BIT: u32 = 0b10000;
    const GUEST_BIT: u32 = 0b10;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { return SUCCESS; }
                if permissions & GROUP_BIT == GROUP_BIT { return SUCCESS; }
                if permissions & GUEST_BIT == GUEST_BIT { return SUCCESS; }
                FAILED
            },
            Err(_) => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file has execute permissions. This function is rather low level because
/// Rust currently does not have a higher level abstraction for obtaining non-standard file modes.
/// To extract the permissions from the mode, the bitwise AND operator will be used and compared
/// with the respective execute bits.
fn file_has_execute_permission(file: Option<&String>) -> i32 {
    const USER_BIT:  u32 = 0b1000000;
    const GROUP_BIT: u32 = 0b1000;
    const GUEST_BIT: u32 = 0b1;

    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => {
                let permissions = metadata.permissions().mode();
                if permissions & USER_BIT == USER_BIT { return SUCCESS; }
                if permissions & GROUP_BIT == GROUP_BIT { return SUCCESS; }
                if permissions & GUEST_BIT == GUEST_BIT { return SUCCESS; }
                FAILED
            },
            Err(_) => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file argument is a socket
fn file_is_socket(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_socket()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file argument is a block device
fn file_is_block_device(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_block_device()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file argument is a character device
fn file_is_character_device(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_char_device()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file exists
fn file_exists(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => evaluate_bool(Path::new(&filepath).exists()),
        None           => SUCCESS
    }
}

/// Exits SUCCESS if the file is a regular file
fn file_is_regular(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_file()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file is a directory
fn file_is_directory(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_dir()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the file is a symbolic link
fn file_is_symlink(file: Option<&String>) -> i32 {
    match file {
        Some(filepath) => match fs::symlink_metadata(filepath) {
            Ok(metadata) => evaluate_bool(metadata.file_type().is_symlink()),
            Err(_)       => FAILED
        },
        None => SUCCESS
    }
}

/// Exits SUCCESS if the string is not empty
fn string_is_nonzero(string: Option<&String>) -> i32 {
    match string {
        Some(string) => evaluate_bool(!string.is_empty()),
        None         => SUCCESS
    }
}

/// Exits SUCCESS if the string is empty
fn string_is_zero(string: Option<&String>) -> i32 {
    match string {
        Some(string) => evaluate_bool(string.is_empty()),
        None         => SUCCESS
    }
}

/// Convert a boolean to it's respective exit code.
fn evaluate_bool(input: bool) -> i32 { if input { SUCCESS } else { FAILED } }

#[test]
fn test_strings() {
    assert_eq!(string_is_zero(Some(&String::from("NOT ZERO"))), FAILED);
    assert_eq!(string_is_zero(Some(&String::from(""))), SUCCESS);
    assert_eq!(string_is_nonzero(Some(&String::from("NOT ZERO"))), SUCCESS);
    assert_eq!(string_is_nonzero(Some(&String::from(""))), FAILED);
}

#[test]
fn test_integers_arguments() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    // Equal To
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-eq"), String::from("10")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-eq"), String::from("5")],
        &mut stdout, &mut stderr), FAILED);

    // Greater Than or Equal To
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-ge"), String::from("10")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-ge"), String::from("5")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-ge"), String::from("10")],
        &mut stdout, &mut stderr), FAILED);

    // Less Than or Equal To
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-le"), String::from("5")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-le"), String::from("10")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-le"), String::from("5")],
        &mut stdout, &mut stderr), FAILED);

    // Less Than
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-lt"), String::from("10")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-lt"), String::from("5")],
        &mut stdout, &mut stderr), FAILED);

    // Greater Than
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-gt"), String::from("5")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-gt"), String::from("10")],
        &mut stdout, &mut stderr), FAILED);

    // Not Equal To
    assert_eq!(evaluate_arguments(vec![String::from("10"), String::from("-ne"), String::from("5")],
        &mut stdout, &mut stderr), SUCCESS);
    assert_eq!(evaluate_arguments(vec![String::from("5"), String::from("-ne"), String::from("5")],
        &mut stdout, &mut stderr), FAILED);
}

#[test]
fn test_file_exists() {
    assert_eq!(file_exists(Some(&String::from("testing/empty_file"))), SUCCESS);
    assert_eq!(file_exists(Some(&String::from("this-does-not-exist"))), FAILED);
}

#[test]
fn test_file_is_regular() {
    assert_eq!(file_is_regular(Some(&String::from("testing/empty_file"))), SUCCESS);
    assert_eq!(file_is_regular(Some(&String::from("testing"))), FAILED);
}

#[test]
fn test_file_is_directory() {
    assert_eq!(file_is_directory(Some(&String::from("testing"))), SUCCESS);
    assert_eq!(file_is_directory(Some(&String::from("testing/empty_file"))), FAILED);
}

#[test]
fn test_file_is_symlink() {
    assert_eq!(file_is_symlink(Some(&String::from("testing/symlink"))), SUCCESS);
    assert_eq!(file_is_symlink(Some(&String::from("testing/empty_file"))), FAILED);
}

#[test]
fn test_file_has_execute_permission() {
    assert_eq!(file_has_execute_permission(Some(&String::from("testing/executable_file"))), SUCCESS);
    assert_eq!(file_has_execute_permission(Some(&String::from("testing/empty_file"))), FAILED);
}

#[test]
fn test_file_size_is_greater_than_zero() {
    assert_eq!(file_size_is_greater_than_zero(Some(&String::from("testing/file_with_text"))), SUCCESS);
    assert_eq!(file_size_is_greater_than_zero(Some(&String::from("testing/empty_file"))), FAILED);
}
