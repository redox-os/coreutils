#![deny(warnings)]
extern crate extra;
extern crate walkdir;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Read, Write, Stderr, StdinLock, StdoutLock};
use std::path::{Path, PathBuf};
use std::process::exit;
use extra::option::OptionalExt;
use walkdir::WalkDir;

const MAN_PAGE: &'static str = r#"NAME
    cp - copy files and directories

SYNOPSIS
    cp [-i | --interactive] [-n | --no-clobber] [-v | --verbose] [-h | --help] [-r | -R | --recursive] SOURCES.. DESTINATION

DESCRIPTION
    Copies files and directories as SOURCES to their respective DESTINATION.

    If the target is a directory, the source will be moved into that directory.

OPTIONS
    -h
    --help
        Display this help information and exit.

    -i
    --interactive
        Prompt before overwriting existing files.

    -n
    --no-clobber
        Do not overwrite existing files.

    -r
    -R
    --recursive
        Recursively copy directories.

    -v
    --verbose
        Print the file changes that have been successfully performed.

AUTHOR
    Written by Michael Murphy.
"#;

/// Contains the sources, target and flags that were given as input arguments.
struct Program {
    sources: Vec<PathBuf>,
    target:  PathBuf,
    flags:   Flags
}

impl Program {
    /// Initialize the program by parsing all of the input arguments.
    fn initialize(stdout: &mut StdoutLock, stderr: &mut Stderr) -> Program {
        // Loop through each argument and check for flags.
        // If the argument is not a flag, add it as a source.
        let mut sources = Vec::new();
        let mut flags = Flags { interactive: false, noclobber: false, recursive: false, verbose: false };
        for argument in env::args().skip(1).collect::<Vec<String>>() {
            match argument.as_str() {
                "-h" | "--help" => {
                    stdout.write(MAN_PAGE.as_bytes()).try(stderr);
                    stdout.flush().try(stderr);
                    exit(0);
                }
                "-i" | "--interactive" => {
                    flags.interactive = true;
                    flags.noclobber = false;
                }
                "-n" | "--no-clobber" => {
                    flags.noclobber = true;
                    flags.interactive = false;
                }
                "-r" | "-R" | "--recursive" => {
                    flags.recursive = true;
                }
                "-v" | "--verbose" => {
                    flags.verbose = true;
                }
                _ => sources.push(PathBuf::from(argument))
            }
        }

        // Check if there are at least two valid arguments were colleced: a source and a target.
        match sources.len() {
            0 => {
                stderr.write(b"missing file operand\nTry 'cp --help' for more information.\n").try(stderr);
                stderr.flush().try(stderr);
                exit(1);
            },
            1 => {
                stderr.write(b"missing target operand after '").try(stderr);
                stderr.write(sources[0].to_string_lossy().as_bytes()).try(stderr);
                stderr.write(b"'\nTry 'mv --help' for more information.\n").try(stderr);
                stderr.flush().try(stderr);
                exit(1);
            }
            _ => ()
        }

        // The target may be popped from the list of arguments because it is the last argument.
        // Because there will always be at least two arguments, the result can be unwrapped.
        let target = sources.pop().unwrap();
        Program { sources: sources, target: target, flags: flags }
    }

    /// Take a list of arguments and attempt to copy each source argument to their respective destination.
    fn execute(&self, stdout: &mut StdoutLock, stderr: &mut Stderr) {
        let mut exit_status = 0i32;
        for source in &self.sources {
            if source.is_dir() {
                if self.flags.recursive {
                    let status = self.copy_directory(&source, stderr, stdout);
                    if exit_status == 0 { exit_status = status; }
                } else {
                    stdout.write(b"omitting directory '").try(stderr);
                    stdout.write(&source.to_string_lossy().as_bytes()).try(stderr);
                    stdout.write(b"'\n").try(stderr);
                    stdout.flush().try(stderr);
                }
            } else {
                let status = self.copy_file(&source, stderr, stdout);
                if exit_status == 0 { exit_status = status; }
            }
        }
        exit(exit_status);
    }

    /// Copy a file from the source path to the target destination.
    fn copy_file(&self, source: &Path, stderr: &mut Stderr, stdout: &mut StdoutLock) -> i32 {
        let target = get_target_path(&self.target, &source, stderr);
        let stdin = io::stdin();
        let stdin = &mut stdin.lock();
        if write_is_allowed(&target, &self.flags, stdout, stdin, stderr) {
            match fs::copy(&source, &target) {
                Ok(_) => {
                    if self.flags.verbose { verbose_print(source, &target, stdout, stderr); }
                    if let Err(message) = fs::remove_file(source) {
                        stderr.write(b"cannot remove file '").try(stderr);
                        stderr.write(source.to_string_lossy().as_bytes()).try(stderr);
                        stderr.write(b"': ").try(stderr);
                        print_error(message, stderr);
                        return 1i32;
                    }
                },
                Err(message) => {
                    stderr.write(b"cannot copy '").try(stderr);
                    stderr.write(source.to_string_lossy().as_bytes()).try(stderr);
                    stderr.write(b"' to '").try(stderr);
                    stderr.write(&target.to_string_lossy().as_bytes()).try(stderr);
                    stderr.write(b"': ").try(stderr);
                    print_error(message, stderr);
                    return 1i32;
                }
            }
        }
        return 0i32;
    }

    /// Copy a source directory and all of it's contents to the target destination.
    fn copy_directory(&self, source: &Path, stderr: &mut Stderr, stdout: &mut StdoutLock) -> i32 {
        let mut exit_status = 0i32;
        for entry in WalkDir::new(source) {
            // Because the target will change for each entry, a mutable PathBuf will be created from the target Path.
            let mut current_target = self.target.clone();
            let entry = match entry {
                Ok(entry) => entry,
                Err(message) => {
                    stderr.write(b"cannot access '").try(stderr);
                    stderr.write(message.path().unwrap().to_string_lossy().as_bytes()).try(stderr);
                    stderr.write(b"': ").try(stderr);
                    stderr.write(message.description().as_bytes()).try(stderr);
                    stderr.write(b"\n").try(stderr);
                    stderr.flush().try(stderr);
                    exit_status = 1;
                    continue
                }
            };
            let entry = entry.path();

            // Pushing an absolute path onto a PathBuf causes the PathBuf to be overwritten.
            // Therefore, we will strip the source path from the entry path.
            if entry.is_absolute() {
                let mut temp = source.to_path_buf();
                if !temp.pop() {
                    stderr.write(b"unable to get parent from '").try(stderr);
                    stderr.write(source.to_string_lossy().as_bytes()).try(stderr);
                    stderr.write(b"'\n").try(stderr);
                    stderr.flush().try(stderr);
                    exit_status = 1;
                    continue
                }
                let suffix = entry.strip_prefix(&temp).unwrap();
                current_target.push(suffix);
            } else {
                current_target.push(&entry);
            }

            // If the entry is a directory, create the directory.
            // If the entry is a file, copy the file.
            let stdin = io::stdin();
            let stdin = &mut stdin.lock();
            if write_is_allowed(&current_target, &self.flags, stdout, stdin, stderr) {
                if entry.is_dir() {
                    match fs::create_dir(&current_target) {
                        Ok(_) => if self.flags.verbose { verbose_print(&entry, &current_target, stdout, stderr); },
                        Err(message) => {
                            stderr.write(b"cannot create directory '").try(stderr);
                            stderr.write(&current_target.to_string_lossy().as_bytes()).try(stderr);
                            stderr.write(b"': ").try(stderr);
                            print_error(message, stderr);
                            exit_status = 1;
                        }
                    }
                } else {
                    match fs::copy(&entry, &current_target) {
                        Ok(_) => {
                            if self.flags.verbose { verbose_print(&entry, &current_target, stdout, stderr); }
                        },
                        Err(message) => {
                            stderr.write(b"cannot copy '").try(stderr);
                            stderr.write(&entry.to_string_lossy().as_bytes()).try(stderr);
                            stderr.write(b"' to '").try(stderr);
                            stderr.write(&current_target.to_string_lossy().as_bytes()).try(stderr);
                            stderr.write(b"': ").try(stderr);
                            print_error(message, stderr);
                            exit_status = 1;
                        }
                    }
                }
            }
        }
        exit_status
    }
}

/// Stores the state of each flag.
struct Flags {
    interactive: bool,
    noclobber:   bool,
    recursive:   bool,
    verbose:     bool,
}

fn main() {
    let stderr = &mut io::stderr();
    let stdout = io::stdout();
    let stdout = &mut stdout.lock();
    Program::initialize(stdout, stderr).execute(stdout, stderr);
}

/// Determines if it is okay to overwrite a file that already exists, if it exists.
///
/// - If the target file exists and the no-clobber flag is set, return false.
/// - If the target file exists and the interactive flag is set, prompt the user if it is okay to overwrite.
/// - Otherwise, this will return true in order to allow writing.
fn write_is_allowed(target: &Path, flags: &Flags, stdout: &mut StdoutLock, stdin: &mut StdinLock, stderr: &mut Stderr) -> bool {
    // Skip to the next source if the target exists and we are not allowed to overwrite it.
    if fs::metadata(&target).is_ok() {
        if target.is_dir() || flags.noclobber {
            return false;
        } else if flags.interactive {
            stdout.write(b"overwrite '").try(stderr);
            stdout.write(target.to_string_lossy().as_bytes()).try(stderr);
            stdout.write(b"'? ").try(stderr);
            stdout.flush().try(stderr);
            let input = &mut String::new();
            stdin.read_line(input).try(stderr);
            if input.chars().next().unwrap() != 'y' { return false; }
        }
    }
    return true;
}

/// Print the message given by an io::Error to stderr.
fn print_error(message: io::Error, stderr: &mut Stderr) {
    stderr.write(message.description().as_bytes()).try(stderr);
    stderr.write(b"\n").try(stderr);
    stderr.flush().try(stderr);
}

/// If verbose mode is enabled, print the action that was successfully performed.
fn verbose_print(source: &Path, target: &Path, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    stdout.write(b"'").try(stderr);
    stdout.write(source.to_string_lossy().as_bytes()).try(stderr);
    stdout.write(b"' -> '").try(stderr);
    stdout.write(target.to_string_lossy().as_bytes()).try(stderr);
    stdout.write(b"'\n").try(stderr);
    stdout.flush().try(stderr);
}

/// Uses the target path and source path to determine the effective target path.
fn get_target_path(target_path: &Path, source: &Path, stderr: &mut Stderr) -> PathBuf {
    let mut target = PathBuf::from(target_path);
    if fs::metadata(target_path).is_ok() && target.is_absolute() && target_path.is_dir() {
        let filename = source.file_name().unwrap_or_default();
        target.push(Path::new(filename));
    } else if &target_path == &Path::new(".") {
        target = get_current_directory(stderr);
        let filename = source.file_name().unwrap_or_default();
        target.push(Path::new(filename));
    } else {
        resolve_target_prefixes(&mut target, stderr);
        if fs::metadata(&target).is_ok() && fs::metadata(&target).unwrap().is_dir() {
            let filename = source.file_name().unwrap_or_default();
            target.push(Path::new(filename));
        }
    }
    target
}

// If the target contains ".." path prefixes, this function will resolve the path.
fn resolve_target_prefixes(path: &mut PathBuf, stderr: &mut Stderr) {
    let mut temp = get_current_directory(stderr);
    for component in path.iter() {
        if component == ".." {
            if !temp.pop() {
                stderr.write(b"unable to get parent from current working directory\n'").try(stderr);
                stderr.flush().try(stderr);
                exit(1);
            }
        } else {
            temp.push(component);
        }
    }
    *path = temp;
}

/// Returns the current directory, if possible.
fn get_current_directory(stderr: &mut Stderr) -> PathBuf {
    match std::env::current_dir() {
        Ok(pathbuf) => pathbuf,
        Err(message) => {
            stderr.write(b"unable to get current working directory: ").try(stderr);
            print_error(message, stderr);
            exit(1);
        }
    }
}
