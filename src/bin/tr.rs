#![deny(warnings)]

extern crate coreutils;
extern crate extra;

use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::io::{self, Stdout, Stderr,  BufRead, Read, Write};

use coreutils::ArgParser;
use extra::io::{fail, WriteExt};

static OK: i32                      = 0;
static INVALID_FLAG: i32            = 1;
static REPLACE_CANNOT_BE_EMPTY: i32 = 2;
static WRITE_ERROR: i32             = 3;

static USAGE: &'static str = r#"usage: tr -c string1  string2
    tr -d string1
    tr -s string1 string2
"#;

const MAN_PAGE: &'static str = /* @MANSTART{tr} */ r#"NAME
    tr - translate characters

SYNOPSIS
    tr [ -cdst ] [ string1 [ string2 ] ]

DESCRIPTION
    Tr copies the standard input to the standard output with
    substitution or deletion of selected characters.  Input
    characters found in string1 are mapped into the correspond-
    ing characters of string2. When string2 is short it is pad-
    ded to the length of string1 by duplicating its last charac-
    ter.  Any combination of the options -cds may be used:

    --complement
    -c   complement the set of characters in string1 with
         respect to the universe of characters whose ASCII codes
         are 01 through 0377

    --delete
    -d   delete all input characters in string1

    --squeeze
    -s   squeeze all strings of repeated output characters that
         are in string2 to single characters

    --truncate
    -t   first truncate string1 to length of string2

    In either string the notation a-b means a range of charac-
    ters from a to b in increasing ASCII order.  The character
    `\' followed by 1, 2 or 3 octal digits stands for the char-
    acter whose ASCII code is given by those digits.  A `\' fol-
    lowed by any other character stands for that character.

    The following example creates a list of all the words in
    `file1' one per line in `file2', where a word is taken to be
    a maximal string of alphabetics.  The second string is
    quoted to protect `\' from the Shell.  012 is the ASCII code
    for newline.

        tr -cs A-Za-z '\012' <file1 >file2

SEE ALSO
    ed(1), ascii(7)

BUGS
    loads.
"#; /* @MANEND */


struct Translation {
    complement:  bool,
    delete:      bool,
    squeeze:     bool,
    truncate:    bool,
    search:      String,
    replace:     String,
    status:      Cell<i32>
}

impl Translation {

    fn print_opts(&self) {
        println!("flags\ncompliment {}\ndelete: {}\nsqueeze {}", self.complement, self.delete, self.squeeze);
        println!("search: {}", self.search);
        println!("replace: {}", self.replace);
    }

    fn truncate(&mut self, input: String, length: usize) -> String {
        // use an iterator just in case we have diacretes or other complex chars
        let mut new_string = "".to_string();
        {
            let mut char_walker = input.chars();
            for _ in 0 .. length {
                let last_char = char_walker.next().unwrap();
                new_string.push(last_char);
            }
        }
        return new_string;
    }

    fn append_or_truncate(&mut self) -> &mut Translation {
        // first decide
        let search_length = self.search.chars().count();
        let replace_length = self.replace.chars().count();

        if replace_length < search_length {
            //build adjust search or replace?
            if self.truncate {
                let old_value = self.search.clone();
                self.search = self.truncate(old_value, replace_length);
            } else {
                // fill replace with it's last char to match search in length
                let lastchar_as_string = self.replace.chars().last().unwrap();
                for _ in replace_length .. search_length {
                    self.replace.push(lastchar_as_string); // do something
                }
            }
        } else if replace_length > search_length {
            // truncate replaces length to search'
            let old_value = self.replace.clone();
            self.replace = self.truncate(old_value, search_length);
        }
        return self;
    }

    fn make_map(&mut self) -> HashMap<char, char> {
        // prereq is that search and replace are now the same length
        return self.search.chars().zip(self.replace.chars()).collect();
    }

    fn get_opts(&mut self, stdout: &mut Stdout, mut stderr: &mut Stderr) -> &mut Translation {
        let mut parser = ArgParser::new(2)
            .add_flag("c", "complement")
            .add_flag("d", "delete")
            .add_flag("s", "squeeze")
            .add_flag("t", "truncate")
            .add_flag("h", "help");
        parser.parse(env::args());
        if let Err(err) = parser.found_invalid() {
            let _ = stderr.write(err.as_bytes());
            self.status.set(INVALID_FLAG);
        } else {
            self.complement = parser.found("complement");
            self.delete = parser.found("delete");
            self.squeeze = parser.found("squeeze");
            self.truncate = parser.found("truncate");

            if parser.found("help") {
                let _ = stdout.write(MAN_PAGE.as_bytes());
            }
            let mut iter = parser.args.iter();
            let mut next = iter.next();
            if next.is_some() {
                self.search = next.unwrap().clone();
                next = iter.next();
                if next.is_some() {
                    self.replace = next.unwrap().clone();
                }
            } else {
                let _ = stderr.write("set of characters to replace is obligatory".as_bytes());
                let _ = stdout.write(MAN_PAGE.as_bytes());
            }
        }
        return self;
    }

    fn check_opts(&mut self) -> &mut Translation {
        if !self.delete && !self.squeeze && self.replace.len() == 0 {
            // big issue
            println!("replace string can not be empty when neither -s nor -d is specified.");
            println!("{}", MAN_PAGE);
            self.status.set(REPLACE_CANNOT_BE_EMPTY);
        }

        return self;
    }

    fn translate<R: Read, W: Write>(& mut self, input: R, output: W,) {
        // do the work
        let map = self.make_map();
        let reader = io::BufReader::new(input);
        let mut writer = io::BufWriter::new(output);

        for line in reader.lines() {
            let line = line.unwrap();
            let chars = line.chars();
            for kar in chars {
                let mut output = kar;
                if map.get(&kar).is_some() {
                    output = *map.get(&kar).unwrap();
                }
                if writer.write_char(output).is_err() {
                    self.status.set(WRITE_ERROR);
                    return;
                }
            }
            if writer.write_char('\n').is_err() {
                self.status.set(WRITE_ERROR);
                return;
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut tr = Translation { complement: false, delete: false, squeeze: false, truncate: false, search: String::new(), replace: String::new(), status: Cell::new(OK)};
    tr.get_opts(&mut stdout,&mut stderr);
    tr.check_opts();
// actually put them somewhere for retrieval by the other parts of the program instead of print
    tr.append_or_truncate();
    if tr.status.get() > 0 {
        tr.print_opts();
        fail(USAGE, &mut stderr);
    }

    // if complement is turned on recreate 'search' to contain the complement of search
    tr.translate(stdin, &mut stdout);
    // open std input
    // open std ouput
// read a char
// if not in search => pass through
// decide what to do
// switching over:
// case either 'delete' switched on or find matching char in 'replace'
// case 'squeeze' switched on
// 
}

#[cfg(test)]
mod tests {
    use super::Translation;
    use super::OK;
    use std::cell::Cell;

    #[test]
    fn append_replace_when_it_is_short() {
        let mut tr = Translation {search: "abcde".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false, truncate: false, status: Cell::new(OK)};
        tr.append_or_truncate();
        assert_eq!("xyzzz", tr.replace);
    }
    #[test]
    fn append_replace_when_it_is_long() {
        let mut tr = Translation {search: "a".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false, truncate: false, status: Cell::new(OK)};
        tr.append_or_truncate();
        assert_eq!("x", tr.replace);
    }
    #[test]
    fn append_replace_when_it_is_exact_in_length() {
        let mut tr = Translation {search: "abc".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false, truncate: false, status: Cell::new(OK)};
        tr.append_or_truncate();
        assert_eq!("xyz", tr.replace);
    }
    #[test]
    fn make_and_check_map() {
        let mut tr = Translation {search: "abc".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false, truncate: false, status: Cell::new(OK)};
        // precondition is that append_or_truncate is called so search and replace have the same length
        let map = tr.make_map();
        assert_eq!(3, map.len());
        assert_eq!(3, map.keys().count());
        assert_eq!(3, map.values().count());
        assert_eq!(Some(&'x'), map.get(&'a'));
        assert_eq!(Some(&'y'), map.get(&'b'));
        assert_eq!(Some(&'z'), map.get(&'c'));
        assert_eq!(None, map.get(&'d'));
    }
    #[test]
    fn make_and_check_maptranslate() {
        let mut tr = Translation {search: "abc".to_string() , replace: "xyz".to_string(), complement: false, delete: false, squeeze: false, truncate: false, status: Cell::new(OK)};
        // precondition is that append_or_truncate is called so search and replace have the same length
        let map = tr.make_map();
        assert_eq!(3, map.len());
        assert_eq!(3, map.keys().count());
        assert_eq!(3, map.values().count());
        assert_eq!(Some(&'x'), map.get(&'a'));
        assert_eq!(Some(&'y'), map.get(&'b'));
        assert_eq!(Some(&'z'), map.get(&'c'));
        assert_eq!(None, map.get(&'d'));
    }
}
