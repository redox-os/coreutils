extern crate arg_parser;
extern crate extra;

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::time::Duration;
use std::io::{self, BufRead, Read, Write};
use std::io::{Seek, SeekFrom};
use std::error::Error;
use arg_parser::ArgParser;
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

    -F
        Like -f, but keep checking if the files exist or have been replaced.

    -s SECONDS
    --sleep-interval SECONDS
        With -f or -F, read at intervals of SECONDS seconds (defaults to 1.0).

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

fn follow<W>(files: Vec<(&str, Option<File>)>, output: W, sleep_interval: Duration, follow_name: bool) -> io::Result<()>
    where W: Write
{
    if files.is_empty() {
        return Ok(());
    }

    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    let mut writer = io::BufWriter::new(output);

    let mut last_updated_filename = files.last().unwrap().0;

    let mut readers = Vec::new();
    for (filename, file_opt) in files {
        let follow_info = if let Some(file) = file_opt {
            let metadata = file.metadata()?;
            let mut reader = io::BufReader::new(file);
            let file_end = reader.seek(SeekFrom::End(0))?;
            Some((reader, file_end, metadata))
        } else {
            None
        };

        readers.push((filename, follow_info));
    }

    let mut buf = Vec::new();

    loop {
        std::thread::sleep(sleep_interval);

        for &mut (filename, ref mut follow_info) in readers.iter_mut() {
            if follow_name {
                match File::open(filename) {
                    Err(ref e) if follow_info.is_some() => {
                        writeln!(stderr, "tail: file '{}' has become inaccessible: {}", filename, e.description())?;
                        *follow_info = None;
                    }
                    Ok(file) => {
                        if let &mut Some((ref mut reader, ref mut seek_pos, ref mut metadata)) = follow_info {
                            let new_metadata = file.metadata()?;

                            if metadata.dev() != new_metadata.dev() ||
                               metadata.ino() != new_metadata.ino() {
                                writeln!(stderr, "tail: file '{}' replaced", filename)?;
                                *metadata = new_metadata;
                                *reader = io::BufReader::new(file);
                                *seek_pos = reader.seek(SeekFrom::Start(0))?;
                            }
                        } else {
                            writeln!(stderr, "tail: file '{}' appeared", filename)?;
                            let metadata = file.metadata()?;
                            let mut reader = io::BufReader::new(file);
                            let seek_pos = reader.seek(SeekFrom::Start(0))?;
                            *follow_info = Some((reader, seek_pos, metadata));
                        }
                    }
                    _ => {}
                }
            }

            if let &mut Some((ref mut reader, ref mut last_seek_pos, _)) = follow_info {
                let seek_pos = reader.seek(SeekFrom::End(0))?;

                if seek_pos != *last_seek_pos {
                    if filename != last_updated_filename {
                        writer.write_all(b"\n")?;
                        print_filename_header(filename, &mut writer)?;
                        last_updated_filename = filename;
                    }

                    if seek_pos < *last_seek_pos {
                        writeln!(stderr, "tail: file '{}' truncated", filename)?;
                        reader.seek(SeekFrom::Start(0))?;
                    } else {
                        reader.seek(SeekFrom::Start(*last_seek_pos))?;
                    }
                    buf.clear();
                    reader.read_to_end(&mut buf)?;
                    writer.write_all(&buf[..])?;
                    writer.flush()?;

                    *last_seek_pos = seek_pos;
                }
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
    let mut parser = ArgParser::new(6)
        .add_opt_default("n", "lines", "10")
        .add_opt("c", "bytes")
        .add_flag(&["h", "help"])
        .add_flag(&["f"])
        .add_flag(&["F"])
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
    if parser.found(&'F') {
        *parser.flag(&'f') = true;
    }
    if let Err(err) = parser.found_invalid() {
        stderr.write_all(err.as_bytes()).try(&mut stderr);
        stderr.flush().try(&mut stderr);
        return;
    }
    let (lines, skip, num): (bool, bool, usize) =
        if let Some(num) = parser.get_opt("lines") {
            (true, num.starts_with("+"), num.trim_start_matches('+').parse().try(&mut stderr))
        }
        else if let Some(num) = parser.get_opt("bytes") {
            (false, num.starts_with("+"), num.trim_start_matches('+').parse().try(&mut stderr))
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
            .map(|filename| {
                let file_opt = match File::open(filename) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        writeln!(stderr, "tail: cannot open file '{}': {}", filename, e.description()).try(&mut stderr);
                        None
                    }
                };
                (filename.as_str(), file_opt)
            })
            .collect::<Vec<(_, _)>>();

        let mut print_newline = false;
        for &(filename, ref file_opt) in &files {
            if let &Some(ref file) = file_opt {
                if file_count > 1 {
                    if print_newline {
                        stdout.write_all(b"\n").try(&mut stderr);
                    }
                    print_newline = true;
                    print_filename_header(filename, &mut stdout).try(&mut stderr);
                }
                tail(file, &mut stdout, lines, skip, num).try(&mut stderr);
            }
        }

        if parser.found(&'f') {
            let follow_name = parser.found(&'F');
            follow(files, &mut stdout, sleep_interval, follow_name).try(&mut stderr);
        }
    }
}
