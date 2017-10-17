#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
#[macro_use]
extern crate coreutils;

use std::fs;
use std::io::{self, BufRead, Read, Write};
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
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

fn head<R: Read, W: Write>(input: R, output: W, lines: bool, num: usize) -> io::Result<()> {
    let mut writer = io::BufWriter::new(output);

    if lines {
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
    } else {
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
    let mut parser = ArgParser::new(3)
        .add_opt_default("n", "lines", "10")
        .add_opt("c", "bytes")
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("head"), MAN_PAGE);

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    if parser.found(&'c') || parser.found("bytes") {
        parser.opt(&'n').clear();
        parser.opt("lines").clear();
    }

    let (lines, num): (bool, usize) =
        if let Some(num) = parser.get_opt("lines") {
            (true, num.parse().try(&mut stderr))
        }
        else if let Some(num) = parser.get_opt("bytes") {
            (false, num.parse().try(&mut stderr))
        }
        else {
            fail("missing argument (number of lines/bytes)", &mut stderr);
        };


    // run the main part
    if parser.args.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        head(stdin, stdout, lines, num).try(&mut stderr);
    } else if parser.args.len() == 1 {
        let file = fs::File::open(&parser.args[0]).try(&mut stderr);
        head(file, stdout, lines, num).try(&mut stderr);
    } else {
        for path in &parser.args {
            let file = fs::File::open(&path).try(&mut stderr);
            stdout.write(b"==> ").try(&mut stderr);
            stdout.write(path.as_bytes()).try(&mut stderr);
            stdout.writeln(b" <==").try(&mut stderr);
            head(file, &mut stdout, lines, num).try(&mut stderr);
        }
    }
}
