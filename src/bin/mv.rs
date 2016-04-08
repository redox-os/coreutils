#![deny(warnings)]
extern crate extra;
extern crate walkdir;

use std::env;
use std::error::Error;
use std::fs::{self, Metadata};
use std::io::{self, BufRead, Read, Write, Stderr, StdinLock, StdoutLock};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use walkdir::WalkDir;

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
        // Metadata from the source and target are required to determine both are on the same device.
        let target_metadata = get_target_metadata(&arguments.target, &source, stderr);
        let target = get_target_path(&arguments.target, &target_metadata, &source, stderr);
        let source_metadata = match get_source_metadata(&source, stderr) {
            Some(metadata) => metadata,
            None => continue // We will skip this source because there was an error.
        };

        // Move the source file or directory to the target path.
        // If the source and target are on the same device, rename the source.
        // If they are on different devices, copy the file or directory.
        if source_metadata.dev() == target_metadata.dev() {
            match fs::rename(&source, &target) {
                Ok(_) => if arguments.flags.verbose { verbose_print(&source, &target, stdout); },
                Err(message) => {
                    let _ = stderr.write(b"cannot rename '");
                    let _ = stderr.write(&source.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"' to '");
                    let _ = stderr.write(&target.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"': ");
                    print_error(message, stderr);
                }
            }
        } else {
            if source_metadata.is_dir() {
                copy_directory(&source, &arguments.target, &arguments.flags, stderr, stdout);
            } else {
                copy_file(&source, target.as_path(), &arguments.flags, stderr, stdout);
            }
        }
    }
}



/// To move a file across devices, the file must first be copied and then deleted.
fn copy_file(source: &Path, target: &Path, flags: &Flags, stderr: &mut Stderr, stdout: &mut StdoutLock) {
    let stdin = io::stdin();
    let stdin = &mut stdin.lock();
    if write_is_allowed(target, flags, stdout, stdin) {
        match fs::copy(&source, &target) {
            Ok(_) => {
                if flags.verbose { verbose_print(&source, &target, stdout); }
                let _ = fs::remove_file(&source);
            },
            Err(message) => {
                let _ = stderr.write(b"cannot copy '");
                let _ = stderr.write(&source.to_string_lossy().as_bytes());
                let _ = stderr.write(b"' to '");
                let _ = stderr.write(&target.to_string_lossy().as_bytes());
                let _ = stderr.write(b"': ");
                print_error(message, stderr);
            }
        }
    }
}

/// While directories on the same device may simply be moved using fs::rename(), cross-device moving of directories is
/// a bit more involved. The walkdir crate was imported to make this easier.
fn copy_directory(source: &Path, target: &Path, flags: &Flags, stderr: &mut Stderr, stdout: &mut StdoutLock) {
    // Keep track of files and directories to be deleted.
    let mut delete_files = Vec::new();
    let mut directory_walk = Vec::new();

    for entry in WalkDir::new(&source) {
        // Because the target will change for each entry, a mutable PathBuf will be created from the target Path.
        let mut current_target = target.to_path_buf();
        let entry = entry.unwrap();
        let entry = entry.path();
        directory_walk.push(entry.to_path_buf());

        // Pushing an absolute path onto a PathBuf causes the PathBuf to be overwritten.
        // Therefore, we will strip the source path from the entry path.
        if entry.is_absolute() {
            let mut temp = source.to_path_buf();
            let _ = temp.pop();
            let suffix = entry.strip_prefix(&temp).unwrap();
            current_target.push(suffix);
        } else {
            current_target.push(&entry);
        }

        // If the entry is a directory, create the directory.
        // If the entry is a file, copy the file.
        let stdin = io::stdin();
        let stdin = &mut stdin.lock();
        if write_is_allowed(&current_target, flags, stdout, stdin) {
            if entry.is_dir() {
                match fs::create_dir(&current_target) {
                    Ok(_) => if flags.verbose { verbose_print(&entry, &current_target, stdout); },
                    Err(message) => {
                        let _ = stderr.write(b"cannot create directory '");
                        let _ = stderr.write(&current_target.to_string_lossy().as_bytes());
                        let _ = stderr.write(b"': ");
                        print_error(message, stderr);
                    }
                }
            } else {
                match fs::copy(&entry, &current_target) {
                    Ok(_) => {
                        if flags.verbose { verbose_print(&entry, &current_target, stdout); }
                        delete_files.push(entry.to_path_buf());
                    },
                    Err(message) => {
                        let _ = stderr.write(b"cannot copy '");
                        let _ = stderr.write(&entry.to_string_lossy().as_bytes());
                        let _ = stderr.write(b"' to '");
                        let _ = stderr.write(&current_target.to_string_lossy().as_bytes());
                        let _ = stderr.write(b"': ");
                        print_error(message, stderr);
                    }
                }
            }
        }
    }

    // Delete files and directories that
    for entry in directory_walk.iter().rev() {
        if entry.is_dir() {
            match fs::remove_dir(&entry) {
                Ok(_) => if flags.verbose {
                    let _ = stdout.write(b"removed directory '");
                    let _ = stdout.write(&entry.to_string_lossy().as_bytes());
                    let _ = stdout.write(b"'\n");
                    let _ = stdout.flush();
                },
                Err(message) => {
                    let _ = stderr.write(b"cannot remove directory '");
                    let _ = stderr.write(&entry.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"': ");
                    print_error(message, stderr);
                }
            }
        } else if delete_files.contains(&entry) {
            match fs::remove_file(&entry) {
                Ok(_) => if flags.verbose {
                    let _ = stdout.write(b"removed '");
                    let _ = stdout.write(&entry.to_string_lossy().as_bytes());
                    let _ = stdout.write(b"'\n");
                    let _ = stdout.flush();
                },
                Err(message) => {
                    let _ = stderr.write(b"cannot remove file '");
                    let _ = stderr.write(&entry.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"': ");
                    print_error(message, stderr);
                }
            }
        }
    }
}

/// Determines if it is okay to overwrite a file that already exists, if it exists.
///
/// - If the target file exists and the no-clobber flag is set, return false.
/// - If the target file exists and the interactive flag is set, prompt the user if it is okay to overwrite.

/// - Otherwise, this will return true in order to allow writing.
fn write_is_allowed(target: &Path, flags: &Flags, stdout: &mut StdoutLock, stdin: &mut StdinLock) -> bool {
    // Skip to the next source if the target exists and we are not allowed to overwrite it.
    if fs::metadata(&target).is_ok() {
        if target.is_dir() || flags.noclobber {
            return false;
        } else if flags.interactive {
            let _ = stdout.write(b"overwrite '");
            let _ = stdout.write(target.to_string_lossy().as_bytes());
            let _ = stdout.write(b"'? ");
            let _ = stdout.flush();
            let input = &mut String::new();
            let _ = stdin.read_line(input);
            if input.chars().next().unwrap() != 'y' { return false; }
        }
    }
    return true;
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
        Err(message) => {
            let _ = stderr.write(b"cannot stat '");
            let _ = stderr.write(source.to_string_lossy().as_bytes());
            let _ = stderr.write(b"': ");
            print_error(message, stderr);
            return None;
        }
    }
}

/// Obtain the metadata from the target argument, if possible.
fn get_target_metadata(target: &Path, source: &Path, stderr: &mut Stderr) -> Metadata {
    match fs::metadata(target) {
        Ok(metadata) => metadata,
        Err(_) => {
            let mut path = PathBuf::from(target);
            if path.is_absolute() {
                let _ = path.pop();
            } else if &path == &Path::new(".") {
                path = get_current_directory(stderr);
            } else {
                resolve_target_prefixes(&mut path, stderr);
                let _ = path.pop();
            }

            match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(message) => {
                    let _ = stderr.write(b"cannot move '");
                    let _ = stderr.write(source.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"' to '");
                    let _ = stderr.write(target.to_string_lossy().as_bytes());
                    let _ = stderr.write(b"': ");
                    print_error(message, stderr);
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
            let _ = temp.pop();
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
            let _ = stderr.write(b"unable to get current working directory: ");
            print_error(message, stderr);
            exit(1);
        }
    }
}

/// Check the input arguments to determine if enough arguments were given.
fn check_arguments(arguments: &Vec<String>, stdout: &mut StdoutLock, stderr: &mut Stderr) -> Arguments {
    let mut sources = Vec::new();

    // Loop through each argument and check for flags.
    // If the argument is not a flag, add it as a source.
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

    // Check if there are at least two valid arguments were colleced: a source and a target.
    match sources.len() {
        0 => {
            let _ = stderr.write(b"missing file operand\nTry 'mv --help' for more information.\n");
            let _ = stderr.flush();
            exit(1);
        },
        1 => {
            let _ = stderr.write(b"missing target operand after '");
            let _ = stderr.write(sources[0].to_string_lossy().as_bytes());
            let _ = stderr.write(b"'\nTry 'mv --help' for more information.\n");
            let _ = stderr.flush();
            exit(1);
        }
        _ => ()
    }

    // The target may be popped from the list of arguments because it is the last argument.
    // Because there will always be at least two arguments, the result can be unwrapped.
    let target = sources.pop().unwrap();
    Arguments { sources: sources, target: target, flags: flags }
}
