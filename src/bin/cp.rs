#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate walkdir;
#[macro_use]
extern crate coreutils;

use std::fs;
use std::io::stderr;
use std::path;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::io::fail;
use extra::option::OptionalExt;

use walkdir::WalkDir;

const MAN_PAGE: &'static str = /* @MANSTART{cp} */ r#"
NAME
    cp - copy files

SYNOPSIS
    cp SOURCE_FILE, ... DESTINATION

DESCRIPTION
    The cp utility copies the contents of the SOURCE_FILE to the DESTINATION. If multiple
    source files are specified, then they are copied to DESTINATION.

OPTIONS
    -h
    --help
        print this message
    -n
    --no-action
        usefull only in combination with '--verbose'
    -v
    --verbose
        print what is being copied
    -r
    --recusive
        if any of the SOURCE_FILEs is a directory recurse into it and copy any content.
        NOTE: it is illegal for any SOURCE_FILE to be a directory if this flag isn't given
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(1)
        .add_flag(&["r", "recursive"])
        .add_flag(&["n", "no-action"])
        .add_flag(&["v", "verbose"])
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("cp"), MAN_PAGE);

    let mut stderr = stderr();
    let recurse = parser.found("recursive");
    let verbose = parser.found("verbose");
    let execute = !parser.found("no-action");

    if parser.args.is_empty() {
        fail("No source argument. Use --help to see the usage.", &mut stderr);
    } else if parser.args.len() == 1 {
        fail("No destination argument. Use --help to see the usage.", &mut stderr);
    } else {
        // This unwrap won't panic since it's been verified not to be empty
        let dst = parser.args.pop().unwrap();
        let dst = path::PathBuf::from(dst);
        if dst.is_file() {
            if parser.args.len() == 2 {
                let src = path::Path::new(&parser.args[0]);
                if src.is_file() {
                    if execute {
                        fs::copy(src, dst).try(&mut stderr);
                    }
                    if verbose {
                        println!("{}", src.display());
                    }
                } else {
                    fail("Attempted to copy a non-file onto a file. Use --help to see the usage.", &mut stderr);
                }
            } else {
                fail("Cannot copy multiple objects onto one location. Use --help to see the usage.", &mut stderr);
            }
        } else if dst.is_dir() {
            for ref arg in parser.args {
                let src = path::Path::new(arg);
                if src.is_dir() {
                    if recurse {
                        for entry in WalkDir::new(arg) {
                            let entry = entry.unwrap();
                            let src = path::Path::new(entry.path());
                            if execute {
                                fs::create_dir(dst.join(src)).try(&mut stderr);
                            }
                            if verbose {
                                println!("{}", src.display());
                            }
                        }
                    } else {
                        fail("Can not move directories without the -r recursive tag. Use --help to see the usage.", &mut stderr);
                    }
                } else if src.is_file() {
                    if execute {
                        fs::copy(src, dst.join(src.file_name().try(&mut stderr))).try(&mut stderr);
                    }
                    if verbose {
                        println!("{}", src.display());
                    }
                } // here we might also want to check for symlink, hardlink, socket, block, char, ...?
            }
        } else {
            fail("No destination found. Use --help to see the usage.", &mut stderr);
        }
    }
}
