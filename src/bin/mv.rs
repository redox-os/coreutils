#![deny(warnings)]
extern crate extra;

use std::env;
use std::error::Error;
use std::fs::{self, Metadata};
use std::io::{self, Read, Write, Stderr, StdoutLock};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use extra::option::OptionalExt;

const MAN_PAGE: &'static str = r#"NAME
    mv - move (rename) files

SYNOPSIS
    mv [-i | --interactive] [-n | --no-clobber] [-v | --verbose] [h | --help] SOURCES.. DESTINATION

DESCRIPTION
    If a source is on the same device as it's respective destination, it will be renamed. If it is on a different device, it will be copied.

    If the target is a directory, the source will be moved into that directory.

OPTIONS
    -h
    --help
        display this help and exit

    -i
    --interactive
        prompt before overwriting existing files

    -n
    --no-clobber
        do not overwrite existing files

    -v
    --verbose
        print the file changes that have been successfully performed

AUTHOR
    Written by Michael Murphy.
"#;

/// Contains the sources, target and flags that were given as input arguments.
struct Arguments {
    sources: Vec<PathBuf>,
    target:  PathBuf,
    flags:   Flags
}

/// Stores the state of each flag.
struct Flags {
    interactive: bool,
    noclobber:   bool,
    verbose:     bool,
}

fn main() {
    let stderr = &mut io::stderr();
    let stdout = io::stdout();
    let stdout = &mut stdout.lock();
    let arguments = env::args().skip(1).collect::<Vec<String>>();
    mv(check_arguments(&arguments, stdout, stderr), stdout, stderr);
}

/// Take a list of arguments and attempt to move each source argument to their respective destination.
fn mv(arguments: Arguments, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    for source in arguments.sources {
        let target_metadata = get_target_metadata(&arguments.target, stderr);
        let target = get_target_path(&arguments.target, &target_metadata, &source, stderr);
        let source_metadata = match get_source_metadata(&source, stderr) {
            Some(metadata) => metadata,
            None => continue // We will skip this source because there was an error.
        };

        // Skip to the next source if the target exists and we are not allowed to overwrite it.
        if fs::metadata(&target).is_ok() {
            if arguments.flags.noclobber {
                continue
            } else if arguments.flags.interactive {
                let _ = stdout.write(b"overwrite '");
                let _ = stdout.write(target.to_string_lossy().as_bytes());
                let _ = stdout.write(b"'? (y/n) ");
                let _ = stdout.flush();
                let mut input = [0; 1];
                let _ = io::stdin().read(&mut input);
                if input[0] != b'y' { continue }
            }
        }

        // Move the source file to the target path.
        if source_metadata.dev() == target_metadata.dev() {
            match fs::rename(&source, &target) {
                Ok(_) => if arguments.flags.verbose { verbose_print(&source, &target, stdout); },
                Err(message) => print_error(message, stderr)
            }
        } else {
            match fs::copy(&source, &target) {
                Ok(_) => {
                    if arguments.flags.verbose { verbose_print(&source, &target, stdout); }
                    fs::remove_file(&source).try(stderr);
                },
                Err(message) => print_error(message, stderr)
            }
        }
    }
}

/// Print the message given by an io::Error to stderr.
fn print_error(message: io::Error, stderr: &mut Stderr) {
    let _ = stderr.write(message.description().as_bytes());
    let _ = stderr.write(b"\n");
    let _ = stderr.flush();
}

/// If verbose mode is enabled, print the action that was successfully performed.
fn verbose_print(source: &Path, target: &Path, stdout: &mut StdoutLock) {
    let _ = stdout.write(b"'");
    let _ = stdout.write(source.to_string_lossy().as_bytes());
    let _ = stdout.write(b"' -> '");
    let _ = stdout.write(target.to_string_lossy().as_bytes());
    let _ = stdout.write(b"'\n");
    let _ = stdout.flush();
}

/// Uses the target name, target metadata and source path to determine the effective target path.
fn get_target_path(target_name: &Path, target_metadata: &Metadata, source: &Path, stderr: &mut Stderr) -> PathBuf {
    let mut target = PathBuf::from(target_name);
    if fs::metadata(target_name).is_ok() && target.is_absolute() && target_metadata.is_dir() {
        let filename = source.file_name().unwrap_or_default();
        target.push(Path::new(filename));
    } else if &target_name == &Path::new(".") {
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

/// Obtain the metadata from the source argument, if possible.
fn get_source_metadata(source: &Path, stderr: &mut Stderr) -> Option<Metadata> {
    match fs::metadata(source) {
        Ok(metadata) => Some(metadata),
        Err(_) => {
            let _ = stderr.write(b"cannot stat '");
            let _ = stderr.write(source.to_string_lossy().as_bytes());
            let _ = stderr.write(b"': No such file or directory\n");
            let _ = stderr.flush();
            return None;
        }
    }
}

/// Obtain the metadata from the target argument, if possible.
fn get_target_metadata(target: &Path, stderr: &mut Stderr) -> Metadata {
    match fs::metadata(target) {
        Ok(metadata) => metadata,
        Err(_) => {
            let mut path = PathBuf::from(target);
            if path.is_absolute() {
                path = path.parent().unwrap().to_path_buf()
            } else if &path == &Path::new(".") {
                path = get_current_directory(stderr)
            } else {
                resolve_target_prefixes(&mut path, stderr) // Handle cases where ".." is used in the path.
            }

            match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => {
                    let _ = stderr.write(b"cannot move '");
                    let _ = stderr.write(target.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"' to '");
                    let _ = stderr.write(target.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"': No such file or directory\n");
                    let _ = stderr.flush();
                    exit(1);
                }
            }
        }
    }
}

// If the target contains ".." path prefixes, this function will resolve the path.
fn resolve_target_prefixes(path: &mut PathBuf, stderr: &mut Stderr) {
    let mut temp = get_current_directory(stderr);
    for component in path.iter() {
        if component == ".." {
            temp = temp.parent().unwrap().to_path_buf();
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
        Err(_) => {
            let _ = stderr.write(b"current working directory is invalid\n");
            let _ = stderr.flush();
            exit(1);
        }
    }
}

/// Check the input arguments to determine if enough arguments were given.
fn check_arguments(arguments: &Vec<String>, stdout: &mut StdoutLock, stderr: &mut Stderr) -> Arguments {
    let mut sources = Vec::new();
    let mut flags = Flags { interactive: false, noclobber: false, verbose: false };
    for argument in arguments {
        match argument.as_str() {
            "-h" | "--help" => {
                let _ = stdout.write(MAN_PAGE.as_bytes());
                let _ = stdout.flush();
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
            "-v" | "--verbose" => {
                flags.verbose = true;
            }
            _ => sources.push(PathBuf::from(argument))
        }
    }

    match arguments.len() {
        0 => {
            let _ = stderr.write(b"missing file operand\n");
            let _ = stderr.flush();
            exit(1);
        },
        1 => {
            let _ = stderr.write(b"missing target operand after '");
            let _ = stderr.write(arguments[1].as_bytes());
            let _ = stderr.flush();
            exit(1);
        }
        _ => ()
    }

    let target = sources.pop().unwrap(); // Cannot fail because the length is at least 2.
    Arguments { sources: sources, target: target, flags: flags }
}
