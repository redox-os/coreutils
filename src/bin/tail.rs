#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::collections::VecDeque;
use std::env;
use std::fs;
use std::time::Duration;
use std::io::{self, BufRead, Read, Write};
use std::io::{Seek, SeekFrom};
use coreutils::ArgParser;
use extra::option::OptionalExt;
use extra::io::fail;

static MAN_PAGE: &'static str = /* @MANSTART{tail} */ r#"
NAME
    tail - output the last part of a file

SYNOPSIS
    tail [[-h | --help] | [[-n | --lines] [+]LINES] | [[-c | --bytes] [+]BYTES]] [FILE ...]

DESCRIPTION
    Print the last 10 lines of each FILE to standard output. If there are no files, read the
    standard input. If there are multiple files, prefix each one with a header containing it's
    name.

OPTIONS
    -h
    --help
        Print this manual page.

    -n LINES
    --lines LINES
        Print the last LINES lines.

    -n +LINES
    --lines +LINES
        Print all but the first LINES lines.

    -c BYTES
    --bytes BYTES
        Print the last BYTES bytes.

    -c +BYTES
    --bytes +BYTES
        Print all but the first BYTES bytes.

    -f
        Follow the files content, that is, continue to read the given files
        and print any change that occurs.

    -s
    --sleep-interval SECONDS
        With -f, read at intervals of SECONDS seconds (defaults to 1.0).

AUTHOR
    Written by Žad Deljkić.
"#; /* @MANEND */

#[derive(Debug)]
pub struct LinesWithEnding<B> {
    buf: B,
}

impl<B: BufRead> Iterator for LinesWithEnding<B> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

fn lines_with_ending<B: BufRead>(reader: B) -> LinesWithEnding<B> where B: Sized {
    LinesWithEnding { buf: reader }
}


fn tail<R: Read, W: Write>(input: R, output: W, lines: bool, skip: bool, num: usize) -> io::Result<()> {
    let mut writer = io::BufWriter::new(output);

    if lines {
        if skip {
            let lines = lines_with_ending(io::BufReader::new(input)).skip(num);

            for line_res in lines {
                match line_res {
                    Ok(line) => writer.write_all(line.as_bytes())?,
                    Err(err) => return Err(err),
                };
            }
        } else {
            let lines = lines_with_ending(io::BufReader::new(input));
            let mut deque = VecDeque::new();

            for line_res in lines {
                match line_res {
                    Ok(line) => {
                        deque.push_back(line);

                        if deque.len() > num {
                            deque.pop_front();
                        }
                    }
                    Err(err) => return Err(err),
                };
            }

            for line in deque {
                try!(writer.write_all(line.as_bytes()));
            }
        }
    } else {
        if skip {
            let bytes = input.bytes().skip(num);

            for byte_res in bytes {
                match byte_res {
                    Ok(byte) => try!(writer.write_all(&[byte])),
                    Err(err) => return Err(err),
                };
            }
        } else {
            let bytes = input.bytes();
            let mut deque = VecDeque::new();

            for byte_res in bytes {
                match byte_res {
                    Ok(byte) => {
                        deque.push_back(byte);

                        if deque.len() > num {
                            deque.pop_front();
                        }
                    }
                    Err(err) => return Err(err),
                };
            }

            for byte in deque {
                try!(writer.write_all(&[byte]));
            }
        }
    }

    Ok(())
}

fn follow<R, W>(inputs: Vec<(&str, R)>, output: W, sleep_interval: Duration) -> io::Result<()>
    where R: Read + Seek, W: Write
{
    if inputs.is_empty() {
        return Ok(());
    }

    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let mut writer = io::BufWriter::new(output);

    let mut last_updated_filename = inputs.last().unwrap().0;

    let mut readers = Vec::new();
    for (filename, input) in inputs {
        let mut reader = io::BufReader::new(input);
        let input_end = reader.seek(SeekFrom::End(0))?;
        readers.push((filename, reader, input_end));
    }

    let mut buf = Vec::new();

    loop {
        std::thread::sleep(sleep_interval);

        for &mut (filename, ref mut reader, ref mut last_input_end) in readers.iter_mut() {
            let input_end = reader.seek(SeekFrom::End(0))?;

            if input_end != *last_input_end {
                if filename != last_updated_filename {
                    writer.write_all(b"\n")?;
                    print_filename_header(filename, &mut writer)?;
                    last_updated_filename = filename;
                }

                if input_end < *last_input_end {
                    stderr.write_all("tail: file ".as_bytes())?;
                    stderr.write_all(filename.as_bytes())?;
                    stderr.write_all(" truncated\n".as_bytes())?;

                    reader.seek(SeekFrom::Start(0))?;
                } else {
                    reader.seek(SeekFrom::Start(*last_input_end))?;
                }
                buf.clear();
                reader.read_to_end(&mut buf)?;
                writer.write_all(&buf[..])?;
                writer.flush()?;

                *last_input_end = input_end;
            }
        }
    }
}

fn print_filename_header<W: Write>(name: &str, output: &mut W) -> io::Result<()> {
    output.write_all(b"==> ")?;
    output.write_all(name.as_bytes())?;
    output.write_all(b" <==\n")?;
    Ok(())
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let mut parser = ArgParser::new(5)
        .add_opt_default("n", "lines", "10")
        .add_opt("c", "bytes")
        .add_flag(&["h", "help"])
        .add_flag(&["f"])
        .add_opt_default("s", "sleep-interval", "1.0");
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }
    if parser.found(&'c') || parser.found("bytes") {
        parser.opt("lines").clear();
    }
    if let Err(err) = parser.found_invalid() {
        stderr.write_all(err.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        return;
    }
    let (lines, skip, num): (bool, bool, usize) =
        if let Some(num) = parser.get_opt("lines") {
            (true, num.starts_with("+"), num.trim_left_matches('+').parse().try(&mut stderr))
        }
        else if let Some(num) = parser.get_opt("bytes") {
            (false, num.starts_with("+"), num.trim_left_matches('+').parse().try(&mut stderr))
        }
        else {
            fail("missing argument (number of lines/bytes)", &mut stderr);
        };

    let sleep_interval = {
        let secs_str = parser.get_opt("sleep-interval").unwrap();
        let secs = match secs_str.parse::<f32>() {
            Ok(secs) if secs > 0.0 => secs,
            _ => {
                let msg = format!("invalid number of seconds '{}'", secs_str);
                fail(&msg, &mut stderr);
            },
        };
        let millis = (secs * 1000.0) as u64;
        Duration::from_millis(millis)
    };

    // run the main part
    let file_count = parser.args.len();

    if file_count == 0 {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();

        tail(&mut stdin, &mut stdout, lines, skip, num).try(&mut stderr);

        if parser.found(&'f') {
            let mut buf = String::new();
            loop {
                buf.clear();
                stdin.read_line(&mut buf).try(&mut stderr);
                stdout.write_all(buf.as_bytes()).try(&mut stderr);
            }
        }
    } else {
        let files = parser.args.iter()
            .map(|filename| (filename.as_str(), fs::File::open(filename).try(&mut stderr)))
            .collect::<Vec<(_, _)>>();

        let mut print_newline = false;
        for &(filename, ref file) in &files {
            if file_count > 1 {
                if print_newline {
                    stdout.write_all(b"\n").try(&mut stderr);
                }
                print_newline = true;
                print_filename_header(filename, &mut stdout).try(&mut stderr);
            }
            tail(file, &mut stdout, lines, skip, num).try(&mut stderr);
        }

        if parser.found(&'f') {
            follow(files, &mut stdout, sleep_interval).try(&mut stderr);
        }
    }
}
