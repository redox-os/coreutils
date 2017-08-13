#![deny(warnings)]

extern crate arg_parser;
extern crate coreutils;
extern crate extra;

use std::env;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::slice;
use std::str::FromStr;

use arg_parser::ArgParser;

use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;


static USAGE: &'static str = r#"usage: cut -b list [-n] [file ...]
    cut -c list [file ...]
    cut -f list [-s] [-d delim] [file ...]
"#;

static MAN_PAGE : &'static str = /* @MANSTART{cut} */ r#"
NAME
    cut -- cut out selected portions of each line of a file

SYNOPSIS
    cut -b list [-n] [file ...]
    cut -c list [file ...]
    cut -f list [-d delim] [-s] [file ...]

DESCRIPTION
    The cut utility cuts out selected portions of each line (as specified by list) from each file
    and writes them to the standard output. If no file arguments are specified, or a file argument
    is a single dash (`-'), cut reads from the standard input. The items specified by list can be
    in terms of column position or in terms of fields delimited by a special character. Column
    numbering starts from 1.

    The list option argument is a comma or whitespace separated set of numbers and/or number
    ranges. Number ranges consist of a number, a dash (`-'), and a second number and select the
    fields or columns from the first number to the second, inclusive. Numbers or number ranges may
    be preceded by a dash, which selects all fields or columns from 1 to the last number. Numbers
    or number ranges may be followed by a dash, which selects all fields or columns from the last
    number to the end of the line. Numbers and number ranges may be repeated, overlapping, and in
    any order. If a field or column is specified multiple times, it will appear only once in the
    output. It is not an error to select fields or columns not present in the input line.

OPTIONS
    -b list
        The list specifies byte positions.

    -c list
        The list specifies character positions.

    -d delim
        Use delim as the field delimiter character instead of the tab character.

    -f list
        The list specifies fields, separated in the input by the field delimiter character (see the
        -d option.) Output fields are separated by a single occurrence of the field delimiter
        character.

    -s
        Suppress lines with no field delimiter characters. Unless specified, lines with no
        delimiters are passed through unmodified.

EXIT STATUS
    The cut utility exits 0 on success, and >0 if an error occurs.

EXAMPLES
    Extract users' login names and shells from the system passwd(5) file as ``name:shell'' pairs:

        cut -d : -f 1,7 /etc/passwd

    Show the names and login times of the currently logged in users:

        who | cut -c 1-16,26-38

AUTHOR
    Written by Hernán E. Grecco.
"#; /* @MANEND */

/// The Selection object.
#[derive(Debug, PartialEq)]
struct Selection {
    /// Indicates which elements are selected.
    selected: Vec<bool>,
    /// True indicates that the selection continues after the range covered by `selected`.
    to_eol: bool,
}

impl Selection {
    /// Creates an iterator that yields selected values from a second iterator.
    fn select_from<I>(&self, values: I) -> SelectFromIter<I>
        where I: Iterator
    {
        SelectFromIter::new(values, self.selected.iter(), self.to_eol)
    }
}

impl FromStr for Selection {
    type Err = ParseSelectionError;
    /// Constructs a Selection object from a list option argument.
    ///
    /// The list option argument is a comma or whitespace separated set of numbers and/or
    /// number ranges.  Number ranges consist of a number, a dash (`-'), and a second number
    /// and select the fields or columns from the first number to the second, inclusive.
    /// Numbers or number ranges may be preceded by a dash, which selects all fields or columns
    /// from 1 to the last number.  Numbers or number ranges may be followed by a dash,
    /// which selects all fields or columns from the last number to the end of the line.
    /// Numbers and number ranges may be repeated, overlapping, and in any order.
    /// If a field or column is specified multiple times, it will appear only once in the output.
    fn from_str(list: &str) -> Result<Self, ParseSelectionError> {

        let empty = usize::from_str("");

        let mut selected = Vec::with_capacity(20);
        let mut to_eol = false;

        for part in list.split(|c| c == ',' || c == ' ') {
            let subparts: Vec<Result<usize, std::num::ParseIntError>> =
                part.split('-').map(|x| usize::from_str(x.trim())).collect();

            // Matching a slice is currently unstable.
            match (subparts.get(0), subparts.get(1)) {
                // Single Number: M
                (Some(&Ok(index)), None) => toggle(&mut selected, index - 1),
                // Range with open begin: -N
                (Some(e), Some(&Ok(end))) if *e == empty => fill(&mut selected, 0, end),
                // Range with open end: M-
                (Some(&Ok(begin)), Some(e)) if *e == empty => {
                    toggle(&mut selected, begin - 1);
                    to_eol = true;
                }
                // Range: M-N
                (Some(&Ok(begin)), Some(&Ok(end))) => fill(&mut selected, begin - 1, end),

                _ => return Err(ParseSelectionError{ part: String::from(part) }),
            }
        }
        Ok(Selection {
            selected: selected,
            to_eol: to_eol,
        })
    }
}

struct ParseSelectionError { part: String }

/// If an unwrap fails, print this message.
impl fmt::Debug for ParseSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "illegal part: {}", self.part)
    }
}

/// Iterator that simultaneoulsy iters over a value and a boolean iterator,
/// yielding a value from the first when the second is True.
/// If `to_eol` is true, then all values from the first will be yielded
/// when the second reaches the end.
struct SelectFromIter<'a, I>
    where I: Iterator
{
    values: I,
    selected: slice::Iter<'a, bool>,
    to_eol: bool,
    consumed: bool,
}

impl<'a, I> SelectFromIter<'a, I> where I: Iterator
{
    fn new(values: I, selected: slice::Iter<'a, bool>, to_eol: bool) -> Self {
        SelectFromIter {
            values: values,
            selected: selected,
            to_eol: to_eol,
            consumed: false,
        }
    }
}

impl<'a, I> Iterator for SelectFromIter<'a, I> where I: Iterator
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        if self.consumed {
            // The selected iterator was consumed ...
            // but we continue yielding values from the values iterator.
            self.values.next()
        } else {
            // The selected iterator is not yet consumed.
            // Consume both iterators and ..
            // - If the value iterator fininishes, finish also SelectFromIter (return None).
            // - If the value is selected, return it.
            // - If the selected iterator finishes:
            //      finish also SelectFromIter if not to_eol;
            //      return the value otherwise and move to the other branch,
            //        yield the rest from value iterator.
            // - If the value is not selected (_), continue with the loop.
            loop {
                let (v, s) = (self.values.next(), self.selected.next());
                if v.is_none() {
                    return None;
                }
                match s {
                    Some(&true) => return v,
                    None => {
                        // The selected iterator finished,
                        if self.to_eol {
                            self.consumed = true;
                            return v;
                        } else {
                            return None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Set a boolean array element to true, extending the array if necessary.
fn toggle(v: &mut Vec<bool>, index: usize) {
    if index + 1 > v.len() {
        v.resize(index + 1, false);
    }
    v[index] = true;
}

/// Fill a range (including first and last element) in a boolean array to true,
/// extending the array if necessary.
fn fill(v: &mut Vec<bool>, begin: usize, end: usize) {
    // This could be more eficient by breaking it up into
    // differente cases and filling with true and false
    // but I do not think is worth it.
    if end > v.len() {
        v.resize(end, false);
    }
    for index in v.iter_mut().take(end).skip(begin) {
        *index = true;
    }
}

/// Write selected bytes from a reader, returning how many bytes were written.
fn cut_bytes<R: Read, W: Write>(input: R, output: W, selection: &Selection) -> io::Result<usize> {
    let reader = io::BufReader::new(input);
    let mut writer = io::BufWriter::new(output);

    let mut count = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let bytes = line.bytes();
        for el in selection.select_from(bytes) {
            count += try!(writer.write(&[el]));
        }
        count += try!(writer.write(b"\n"));
    }
    Ok(count)
}

/// Write selected characters from a reader, returning how many bytes were written.
fn cut_characters<R: Read, W: Write>(input: R,
                                     output: W,
                                     selection: &Selection)
                                     -> io::Result<usize> {
    let reader = io::BufReader::new(input);
    let mut writer = io::BufWriter::new(output);

    let mut count = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let chars = line.chars();
        for el in selection.select_from(chars) {
            count += try!(writer.write_char(el));
        }
        count += try!(writer.write(b"\n"));
    }
    Ok(count)
}

/// Write selected fields from a reader, returning how many bytes were written.
///
/// The delimiter is also printed when there are more than 1 fields.
///
/// # arguments
///
/// * `delimiter` - Indicates how the fields are delimited
/// * `skip_if_missing` - If true, lines not containing the field delimiter will be skipped.
fn cut_fields<R: Read, W: Write>(input: R,
                                 output: W,
                                 selection: &Selection,
                                 delimiter: &str,
                                 skip_if_missing: bool)
                                 -> io::Result<usize> {
    let reader = io::BufReader::new(input);
    let mut writer = io::BufWriter::new(output);

    let mut count = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        if !line.contains(delimiter) {
            if !skip_if_missing {
                count += try!(writer.write(line.as_bytes()));
                count += try!(writer.write(b"\n"));
            }
            continue;
        }
        let fields = line.split(delimiter);

        // The first element is separated as a way to include the delimiter
        // in the output if there are more than two elements.
        let mut it = selection.select_from(fields);
        if let Some(el) = it.next() {
            count += try!(writer.write(el.as_bytes()));
        } else {
            continue;
        }
        for el in it {
            count += try!(writer.write(delimiter.as_bytes()));
            count += try!(writer.write(el.as_bytes()));
        }
        count += try!(writer.write(b"\n"));
    }
    Ok(count)
}


// Operating mode for cut.
#[derive(Debug, PartialEq)]
enum Mode {
    Bytes,
    Characters,
    Fields,
}


fn main() {
    // Arguments.
    let mut mode = None;
    let mut delimiter = None;
    let mut skip_if_missing = None;
    let mut list = String::new();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut parser = ArgParser::new(6)
        .add_flag(&["h", "help"])
        .add_flag(&["s"])
        .add_opt("b", "")
        .add_opt("c", "")
        .add_opt("f", "")
        .add_opt("d", "");
    parser.parse(env::args());

    if parser.found("help") {
        let _ = stdout.write(MAN_PAGE.as_bytes());
        return;
    }

    if parser.found(&'s') {
        skip_if_missing = Some(true);
    }

    if parser.found(&'b') {
        if mode.is_some() {
            fail(USAGE, &mut stderr);
        }
        mode = Some(Mode::Bytes);
        list = parser.get_opt(&'b').unwrap();
    }

    if parser.found(&'c') {
        if mode.is_some() {
            fail(USAGE, &mut stderr);
        }
        mode = Some(Mode::Characters);
        list = parser.get_opt(&'c').unwrap();
    }

    if parser.found(&'f') {
        if mode.is_some() {
            fail(USAGE, &mut stderr);
        }
        mode = Some(Mode::Fields);
        list = parser.get_opt(&'f').unwrap();
    }

    if parser.found(&'d') {
        let dlm = parser.get_opt(&'d').unwrap();
        if dlm.chars().count() != 1 {
            fail("bad delimiter", &mut stderr)
        }
        delimiter = Some(dlm);
    }

    let paths = parser.args;

    let mode = mode.fail(USAGE, &mut stderr);
    let selection = match Selection::from_str(&list) {
        Ok(selection) => selection,
        Err(_) => fail("illegal list value", &mut stderr)
    };

    if mode != Mode::Fields && (delimiter.is_some() || skip_if_missing.is_some()) {
        fail(USAGE, &mut stderr);
    }

    let delimiter = delimiter.unwrap_or("\t".into());
    let skip_if_missing = skip_if_missing.unwrap_or(false);

    if paths.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let _ = match mode {
                     Mode::Bytes => cut_bytes(stdin, &mut stdout, &selection),
                     Mode::Characters => cut_characters(stdin, &mut stdout, &selection),
                     Mode::Fields => {
                         cut_fields(stdin, &mut stdout, &selection, &delimiter, skip_if_missing)
                     }
                 }
                 .try(&mut stderr);
    } else {
        for path in paths {
            let file = fs::File::open(&path).try(&mut stderr);
            let _ = match mode {
                         Mode::Bytes => cut_bytes(file, &mut stdout, &selection),
                         Mode::Characters => cut_characters(file, &mut stdout, &selection),
                         Mode::Fields => {
                             cut_fields(file, &mut stdout, &selection, &delimiter, skip_if_missing)
                         }
                     }
                     .try(&mut stderr);
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io;
    use std::str::FromStr;

    use super::{cut_characters, cut_bytes, cut_fields, Selection};

    const TXT: &'static str = "copies of\nfurnished\nabcdefges\nThe above\ncopies or\n";
    const CSV: &'static str = "copi,es,of\nfurn,ish,ed\nabcdefges\nThe,ab,ove\ncop,ies,or\n";

    fn extract_bytes(list: &str) -> Vec<u8> {
        let selection = Selection::from_str(list).unwrap();
        let mut reader = io::Cursor::new(TXT);
        let mut buf = io::Cursor::new(Vec::new());
        let _ = cut_bytes(&mut reader, &mut buf, &selection);
        buf.into_inner()
    }

    fn extract_char(list: &str) -> Vec<u8> {
        let selection = Selection::from_str(list).unwrap();
        let mut reader = io::Cursor::new(TXT);
        // let mut buf = io::Cursor::new(Vec::new());
        let mut buf = io::Cursor::new(Vec::new());
        let _ = cut_characters(&mut reader, &mut buf, &selection);
        buf.into_inner()
    }

    fn extract_fields(list: &str, skip_if_missing: bool) -> Vec<u8> {
        let selection = Selection::from_str(list).unwrap();
        let mut reader = io::Cursor::new(CSV);
        // let mut buf = io::Cursor::new(Vec::new());
        let mut buf = io::Cursor::new(Vec::new());
        let _ = cut_fields(&mut reader, &mut buf, &selection, ",", skip_if_missing);
        buf.into_inner()
    }

    #[test]
    fn parse() {
        assert_eq!(Selection::from_str("1").unwrap(),
                   Selection {
                       selected: vec![true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("1 3").unwrap(),
                   Selection {
                       selected: vec![true, false, true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("3").unwrap(),
                   Selection {
                       selected: vec![false, false, true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("1,3").unwrap(),
                   Selection {
                       selected: vec![true, false, true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("1-").unwrap(),
                   Selection {
                       selected: vec![true],
                       to_eol: true,
                   });
        assert_eq!(Selection::from_str("-3").unwrap(),
                   Selection {
                       selected: vec![true, true, true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("1-3").unwrap(),
                   Selection {
                       selected: vec![true, true, true],
                       to_eol: false,
                   });
        assert_eq!(Selection::from_str("2-4,6-").unwrap(),
                   Selection {
                       selected: vec![false, true, true, true, false, true],
                       to_eol: true,
                   });
        assert_eq!(Selection::from_str("2-4,6-").unwrap(),
                   Selection {
                       selected: vec![false, true, true, true, false, true],
                       to_eol: true,
                   });
    }

    #[test]
    fn parse_err() {
        assert!(Selection::from_str("").is_err());
        assert!(Selection::from_str("X").is_err());
        assert!(Selection::from_str("1;5").is_err());
        assert!(Selection::from_str("1,3,5-X ").is_err());
    }


    #[test]
    fn simple_bytes_chars() {
        assert_eq!(extract_bytes("1-3"), "cop\nfur\nabc\nThe\ncop\n".as_bytes());
        assert_eq!(extract_char("1-3"), "cop\nfur\nabc\nThe\ncop\n".as_bytes());
        assert_eq!(extract_bytes("1-"), TXT.as_bytes());
        assert_eq!(extract_char("1-"), TXT.as_bytes());
        assert_eq!(extract_bytes("2-4"), "opi\nurn\nbcd\nhe \nopi\n".as_bytes());
        assert_eq!(extract_char("2-4"), "opi\nurn\nbcd\nhe \nopi\n".as_bytes());
    }

    #[test]
    fn fields() {
        assert_eq!(extract_fields("1", false),
                   "copi\nfurn\nabcdefges\nThe\ncop\n".as_bytes());
        assert_eq!(extract_fields("1", true),
                   "copi\nfurn\nThe\ncop\n".as_bytes());
        assert_eq!(extract_fields("1,3", true),
                   "copi,of\nfurn,ed\nThe,ove\ncop,or\n".as_bytes());
        assert_eq!(extract_fields("1-", false), CSV.as_bytes());
    }
}
