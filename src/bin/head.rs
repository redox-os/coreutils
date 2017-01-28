#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use coreutils::ArgParser;
use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;

static MAN_PAGE: &'static str = /* @MANSTART{head} */ r#"
NAME
    head - output the first part of a file

SYNOPSIS
    head [[-h | --help] | [[-n | --lines] LINES] | [[-c | --bytes] BYTES]] [FILE ...]

DESCRIPTION
    Print the first 10 lines of each FILE to standard output. If there are no files, read the
    standard input. If there are multiple files, prefix each one with a header containing it's
    name.

OPTIONS
    -h
    --help
        Print this manual page.

    -n LINES
    --lines LINES
        Print the first LINES lines.

    -c BYTES
    --bytes BYTES
        Print the first BYTES bytes.

AUTHOR
    Written by Žad Deljkić.
"#; /* @MANEND */

fn head<R: Read, W: Write>(input: R, output: W, num: usize, parser: &ArgParser) -> io::Result<()> {
    let mut writer = io::BufWriter::new(output);

    if parser.found(&'n') || parser.found("lines") {
        let lines = io::BufReader::new(input).lines().take(num);

        for line_res in lines {
            match line_res {
                Ok(mut line) => {
                    line.push('\n');
                    try!(writer.write_all(line.as_bytes()));
                }
                Err(err) => return Err(err),
            };
        }
    }
    else if parser.found(&'c') || parser.found("bytes") {
        let bytes = input.bytes().take(num);

        for byte_res in bytes {
            match byte_res {
                Ok(byte) => try!(writer.write_all(&[byte])),
                Err(err) => return Err(err),
            };
        }
    }

    Ok(())
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(3)
        .add_opt_default("n", "lines", "10")
        .add_opt("c", "bytes")
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }
    if parser.found(&'c') || parser.found("bytes") {
        parser.opt(&'n').clear();
        parser.opt("lines").clear();
    }
    if let Err(err) = parser.found_invalid() {
        stderr.write_all(err.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        return;
    }

    let num: usize = 
        if let Some(num) = parser.get_opt("lines") {
            num.parse().try(&mut stderr)
        }
        else if let Some(num) = parser.get_opt("bytes") {
            num.parse().try(&mut stderr)
        }
        else {
            fail("missing argument (number of lines/bytes)", &mut stderr);
        };


    // run the main part
    if parser.args.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        head(stdin, stdout, num, &parser).try(&mut stderr);
    } else if parser.args.len() == 1 {
        let file = fs::File::open(&parser.args[0]).try(&mut stderr);
        head(file, stdout, num, &parser).try(&mut stderr);
    } else {
        for path in &parser.args {
            let file = fs::File::open(&path).try(&mut stderr);
            stdout.write(b"==> ").try(&mut stderr);
            stdout.write(path.as_bytes()).try(&mut stderr);
            stdout.writeln(b" <==").try(&mut stderr);
            head(file, &mut stdout, num, &parser).try(&mut stderr);
        }
    }
}
