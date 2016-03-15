#![deny(warnings)]

extern crate coreutils;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use coreutils::extra::{OptionalExt, fail};

static MAN_PAGE: &'static str = r#"NAME
    head - output the first part of a file

SYNOPSIS
    head [[-h | --help] | [[-n | --lines] LINES] | [[-c | --bytes] BYTES]] [FILE ...]

DESCRIPTION
    Print the first 10 lines of each FILE to standard output. If there are no files, read the standard input. If there are multiple files, prefix each one with a header containing it's name.

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
"#;

#[derive(Copy, Clone)]
struct Options {
    /// true if outputing lines, false if outputing bytes
    lines: bool,
    /// number of lines/bytes specified
    num: usize,
}

fn head<R: Read, W: Write>(input: R, output: W, opts: Options) -> io::Result<()> {
    let mut writer = io::BufWriter::new(output);

    if opts.lines {
        let lines = io::BufReader::new(input).lines().take(opts.num);

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
        let bytes = input.bytes().take(opts.num);

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
    // default options
    let mut opts = Options {
        lines: true,
        num: 10,
    };
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut args = env::args().skip(1);
    let mut paths: Vec<String> = Vec::new();

    // parse options
    while let Some(arg) = args.next() {
        if arg.starts_with('-') {
            match arg.as_str() {
                "-h" | "--help" => {
                    stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
                    return;
                }
                "-n" | "--lines" => opts.lines = true,
                "-c" | "--bytes" => opts.lines = false,
                _ => fail("invalid option", &mut stderr),
            }

            if let Some(arg) = args.next() {
                opts.num = arg.parse::<usize>().try(&mut stderr);
            } else {
                fail("missing argument (number of lines/bytes)", &mut stderr);
            }
        } else {
            paths.push(arg);
        }
    }

    // run the main part
    if paths.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        head(stdin, stdout, opts).try(&mut stderr);
    } else if paths.len() == 1 {
        let file = fs::File::open(&paths[0]).try(&mut stderr);
        head(file, stdout, opts).try(&mut stderr);
    } else {
        for path in paths {
            let file = fs::File::open(&path).try(&mut stderr);
            writeln!(&mut stdout, "==> {} <==", path).try(&mut stderr);
            head(file, &mut stdout, opts).try(&mut stderr);
            writeln!(&mut stdout, "").try(&mut stderr);
        }
    }
}
